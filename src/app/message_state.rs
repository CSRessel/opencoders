use opencode_sdk::models::{SessionMessages200ResponseInner, Message, Part};
use std::collections::{HashMap, HashSet};
use std::time::SystemTime;

#[derive(Debug, Clone, PartialEq)]
pub struct MessageState {
    // Indexed storage for efficient updates
    messages: HashMap<String, MessageContainer>, // message_id -> MessageContainer
    message_order: Vec<String>, // Ordered list of message IDs for display
    
    // Current session context
    current_session_id: Option<String>,
    
    // Streaming state tracking
    streaming_messages: HashSet<String>, // message IDs currently streaming
}

#[derive(Debug, Clone, PartialEq)]
pub struct MessageContainer {
    pub info: Message, // User or Assistant message info
    pub parts: HashMap<String, Part>, // part_id -> Part for efficient updates
    pub part_order: Vec<String>, // Ordered list of part IDs
    pub is_streaming: bool,
    pub last_updated: SystemTime,
    pub printed_to_stdout: bool, // Track if this message has been printed to stdout
}

impl MessageState {
    pub fn new() -> Self {
        Self {
            messages: HashMap::new(),
            message_order: Vec::new(),
            current_session_id: None,
            streaming_messages: HashSet::new(),
        }
    }

    pub fn set_session_id(&mut self, session_id: Option<String>) {
        if self.current_session_id != session_id {
            // Clear messages when switching sessions
            self.clear();
            self.current_session_id = session_id;
        }
    }

    pub fn clear(&mut self) {
        self.messages.clear();
        self.message_order.clear();
        self.streaming_messages.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    pub fn load_messages(&mut self, messages: Vec<GetSessionByIdMessage200ResponseInner>) {
        self.clear();
        
        for msg_container in messages {
            let message_id = self.extract_message_id(&msg_container.info);
            
            let mut parts_map = HashMap::new();
            let mut part_order = Vec::new();
            
            for part in msg_container.parts {
                let part_id = self.extract_part_id(&part);
                part_order.push(part_id.clone());
                parts_map.insert(part_id, part);
            }
            
            let container = MessageContainer {
                info: *msg_container.info,
                parts: parts_map,
                part_order,
                is_streaming: false,
                last_updated: SystemTime::now(),
                printed_to_stdout: false, // Loaded messages should be printed in inline mode
            };
            
            self.messages.insert(message_id.clone(), container);
            self.message_order.push(message_id);
        }
    }

    pub fn update_message(&mut self, message_info: Message) -> bool {
        let message_id = self.extract_message_id(&message_info);
        
        // Only process messages for current session
        if let Some(current_session) = &self.current_session_id {
            let message_session_id = self.extract_session_id_from_message(&message_info);
            if &message_session_id != current_session {
                return false;
            }
        }
        
        match self.messages.get_mut(&message_id) {
            Some(container) => {
                // Update existing message
                container.info = message_info;
                container.last_updated = SystemTime::now();
                true
            }
            None => {
                // Create new message
                let container = MessageContainer {
                    info: message_info,
                    parts: HashMap::new(),
                    part_order: Vec::new(),
                    is_streaming: true, // New messages start as streaming
                    last_updated: SystemTime::now(),
                    printed_to_stdout: false, // New messages haven't been printed yet
                };
                
                self.messages.insert(message_id.clone(), container);
                self.message_order.push(message_id.clone());
                self.streaming_messages.insert(message_id);
                true
            }
        }
    }

    pub fn update_message_part(&mut self, part: Part) -> bool {
        let part_id = self.extract_part_id(&part);
        let message_id = self.extract_message_id_from_part(&part);
        
        // Only process parts for current session
        if let Some(current_session) = &self.current_session_id {
            let part_session_id = self.extract_session_id_from_part(&part);
            if &part_session_id != current_session {
                return false;
            }
        }
        
        if let Some(container) = self.messages.get_mut(&message_id) {
            let is_new_part = !container.parts.contains_key(&part_id);
            
            if is_new_part {
                container.part_order.push(part_id.clone());
            }
            
            container.parts.insert(part_id, part);
            container.last_updated = SystemTime::now();
            
            // Mark as streaming if this is a new or updated part
            container.is_streaming = true;
            self.streaming_messages.insert(message_id);
            
            true
        } else {
            false
        }
    }

    pub fn remove_message(&mut self, session_id: &str, message_id: &str) -> bool {
        // Only process removals for current session
        if let Some(current_session) = &self.current_session_id {
            if session_id != current_session {
                return false;
            }
        }
        
        if self.messages.remove(message_id).is_some() {
            self.message_order.retain(|id| id != message_id);
            self.streaming_messages.remove(message_id);
            true
        } else {
            false
        }
    }

    pub fn mark_message_complete(&mut self, message_id: &str) {
        if let Some(container) = self.messages.get_mut(message_id) {
            container.is_streaming = false;
            self.streaming_messages.remove(message_id);
        }
    }

    pub fn get_all_message_containers(&self) -> Vec<&MessageContainer> {
        self.message_order
            .iter()
            .filter_map(|message_id| self.messages.get(message_id))
            .collect()
    }

    pub fn is_message_streaming(&self, message_id: &str) -> bool {
        self.streaming_messages.contains(message_id)
    }

    pub fn get_streaming_message_count(&self) -> usize {
        self.streaming_messages.len()
    }

    pub fn get_messages_needing_stdout_print(&self) -> Vec<String> {
        let mut messages_to_print = Vec::new();
        
        for message_id in &self.message_order {
            if let Some(container) = self.messages.get(message_id) {
                if !container.printed_to_stdout {
                    // Extract text content from message parts for printing
                    let mut text_content = String::new();
                    
                    for part_id in &container.part_order {
                        if let Some(part) = container.parts.get(part_id) {
                            match part {
                                Part::Text(text_part) => {
                                    if !text_content.is_empty() {
                                        text_content.push(' ');
                                    }
                                    text_content.push_str(&text_part.text);
                                }
                                _ => {} // Only print text parts for now
                            }
                        }
                    }
                    
                    if !text_content.is_empty() {
                        messages_to_print.push(text_content);
                    }
                }
            }
        }
        
        messages_to_print
    }

    pub fn mark_messages_printed_to_stdout(&mut self, count: usize) {
        let mut marked = 0;
        
        for message_id in &self.message_order {
            if marked >= count {
                break;
            }
            
            if let Some(container) = self.messages.get_mut(message_id) {
                if !container.printed_to_stdout {
                    container.printed_to_stdout = true;
                    marked += 1;
                }
            }
        }
    }

    pub fn has_messages_needing_stdout_print(&self) -> bool {
        self.message_order.iter().any(|message_id| {
            self.messages.get(message_id)
                .map(|container| !container.printed_to_stdout)
                .unwrap_or(false)
        })
    }

    pub fn get_message_containers_for_rendering(&self) -> Vec<&MessageContainer> {
        self.message_order
            .iter()
            .filter_map(|message_id| {
                self.messages.get(message_id).filter(|container| !container.printed_to_stdout)
            })
            .collect()
    }

    // Helper methods to extract IDs from different message types
    fn extract_message_id(&self, message: &Message) -> String {
        match message {
            Message::User(user_msg) => user_msg.id.clone(),
            Message::Assistant(assistant_msg) => assistant_msg.id.clone(),
        }
    }

    fn extract_session_id_from_message(&self, message: &Message) -> String {
        match message {
            Message::User(user_msg) => user_msg.session_id.clone(),
            Message::Assistant(assistant_msg) => assistant_msg.session_id.clone(),
        }
    }

    fn extract_part_id(&self, part: &Part) -> String {
        match part {
            Part::Text(text_part) => text_part.id.clone(),
            Part::Tool(tool_part) => tool_part.id.clone(),
            Part::File(file_part) => file_part.id.clone(),
            Part::StepStart(step_part) => step_part.id.clone(),
            Part::StepFinish(step_part) => step_part.id.clone(),
            Part::Snapshot(snapshot_part) => snapshot_part.id.clone(),
        }
    }

    fn extract_message_id_from_part(&self, part: &Part) -> String {
        match part {
            Part::Text(text_part) => text_part.message_id.clone(),
            Part::Tool(tool_part) => tool_part.message_id.clone(),
            Part::File(file_part) => file_part.message_id.clone(),
            Part::StepStart(step_part) => step_part.message_id.clone(),
            Part::StepFinish(step_part) => step_part.message_id.clone(),
            Part::Snapshot(snapshot_part) => snapshot_part.message_id.clone(),
        }
    }

    fn extract_session_id_from_part(&self, part: &Part) -> String {
        match part {
            Part::Text(text_part) => text_part.session_id.clone(),
            Part::Tool(tool_part) => tool_part.session_id.clone(),
            Part::File(file_part) => file_part.session_id.clone(),
            Part::StepStart(step_part) => step_part.session_id.clone(),
            Part::StepFinish(step_part) => step_part.session_id.clone(),
            Part::Snapshot(snapshot_part) => snapshot_part.session_id.clone(),
        }
    }
}

impl Default for MessageState {
    fn default() -> Self {
        Self::new()
    }
}