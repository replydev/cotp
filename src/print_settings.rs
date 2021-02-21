pub struct PrintSettings {
    max_id: usize,
    max_issuer: usize,
    max_label: usize,
    max_code: usize,
}

impl PrintSettings {
    pub fn new() -> PrintSettings{
        // set the length of id, issuer, label, and code words
        PrintSettings {
            max_id: 2,
            max_issuer: 6,
            max_label: 5,
            max_code: 4,
        }
    }

    pub fn set_max_id(&mut self, id: usize) -> &mut PrintSettings{
        self.max_id = id;
        self
    }

    pub fn get_max_id(&self) -> usize {
        self.max_id
    }

    pub fn set_max_issuer(&mut self, max_issuer: usize) -> &mut PrintSettings{
        self.max_issuer = max_issuer;
        self
    }

    pub fn get_max_issuer(&self) -> usize {
        self.max_issuer
    }

    pub fn set_max_label(&mut self, max_label: usize) -> &mut PrintSettings{
        self.max_label = max_label;
        self
    }

    pub fn get_max_label(&self) -> usize {
        self.max_label
    }

    pub fn set_max_code(&mut self, max_code: usize) -> &mut PrintSettings{
        self.max_code = max_code;
        self
    }

    pub fn get_max_code(&self) -> usize {
        self.max_code
    }

    pub fn check_other(&mut self,other: &PrintSettings){
        if other.get_max_id() > self.get_max_id() {
            self.set_max_id(other.get_max_id());
        }
        if other.get_max_issuer() > self.get_max_issuer() {
            self.set_max_issuer(other.get_max_issuer());
        }
        if other.get_max_label() > self.get_max_label() {
            self.set_max_label(other.get_max_label());
        }
        if other.get_max_code() > self.get_max_code() {
            self.set_max_code(other.get_max_code());
        }
    }

    pub fn get_width(&self) -> usize {
        self.max_id + 2 + self.max_issuer + 2 + self.max_label + 2 + self.max_code + 2 + 2
    }
}
