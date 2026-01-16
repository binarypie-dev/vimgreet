use zeroize::Zeroize;

#[derive(Default, Clone)]
pub struct InputBuffer {
    content: String,
    cursor: usize,
    masked: bool,
}

impl InputBuffer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn masked() -> Self {
        Self {
            content: String::new(),
            cursor: 0,
            masked: true,
        }
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    pub fn len(&self) -> usize {
        self.content.chars().count()
    }

    pub fn display(&self, mask_char: char) -> String {
        if self.masked {
            mask_char.to_string().repeat(self.len())
        } else {
            self.content.clone()
        }
    }

    pub fn insert(&mut self, c: char) {
        let byte_pos = self.cursor_byte_position();
        self.content.insert(byte_pos, c);
        self.cursor += 1;
    }

    pub fn delete_back(&mut self) -> bool {
        if self.cursor > 0 {
            self.cursor -= 1;
            let byte_pos = self.cursor_byte_position();
            let next_byte_pos = self.content[byte_pos..]
                .char_indices()
                .nth(1)
                .map(|(i, _)| byte_pos + i)
                .unwrap_or(self.content.len());
            self.content.drain(byte_pos..next_byte_pos);
            true
        } else {
            false
        }
    }

    pub fn delete_forward(&mut self) -> bool {
        let len = self.len();
        if self.cursor < len {
            let byte_pos = self.cursor_byte_position();
            let next_byte_pos = self.content[byte_pos..]
                .char_indices()
                .nth(1)
                .map(|(i, _)| byte_pos + i)
                .unwrap_or(self.content.len());
            self.content.drain(byte_pos..next_byte_pos);
            true
        } else {
            false
        }
    }

    pub fn move_left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn move_right(&mut self) {
        if self.cursor < self.len() {
            self.cursor += 1;
        }
    }

    pub fn move_start(&mut self) {
        self.cursor = 0;
    }

    pub fn move_end(&mut self) {
        self.cursor = self.len();
    }

    pub fn clear(&mut self) {
        self.content.zeroize();
        self.content.clear();
        self.cursor = 0;
    }

    pub fn set(&mut self, value: &str) {
        self.content.zeroize();
        self.content = value.to_string();
        self.cursor = self.len();
    }

    fn cursor_byte_position(&self) -> usize {
        self.content
            .char_indices()
            .nth(self.cursor)
            .map(|(i, _)| i)
            .unwrap_or(self.content.len())
    }
}

impl Drop for InputBuffer {
    fn drop(&mut self) {
        if self.masked {
            self.content.zeroize();
        }
    }
}
