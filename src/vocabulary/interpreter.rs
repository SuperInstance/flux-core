use crate::bytecode::assembler::Assembler;
use crate::vm::interpreter::Interpreter as VmInterpreter;
use regex::Regex;

/// A single vocabulary entry that maps a natural language pattern to assembly code
#[derive(Debug, Clone)]
pub struct VocabEntry {
    /// Regex pattern to match against natural language input
    pub pattern: String,
    /// Assembly template with {n} placeholders for captured groups
    pub assembly_template: String,
    /// Register that will contain the result
    pub result_reg: u8,
    /// Human-readable name for this entry
    pub name: String,
}

impl VocabEntry {
    pub fn new(pattern: &str, assembly_template: &str, result_reg: u8, name: &str) -> Self {
        Self {
            pattern: pattern.to_string(),
            assembly_template: assembly_template.to_string(),
            result_reg,
            name: name.to_string(),
        }
    }

    /// Match the pattern against input text and substitute captured groups into template
    pub fn match_and_substitute(&self, input: &str) -> Option<String> {
        let re = Regex::new(&self.pattern).ok()?;
        let captures = re.captures(input)?;

        let mut result = self.assembly_template.clone();
        for i in 0.. {
            match captures.get(i + 1) {
                Some(matched) => {
                    let placeholder = format!("{{{}}}", i);
                    result = result.replace(&placeholder, matched.as_str());
                }
                None => break,
            }
        }

        Some(result)
    }
}

/// A collection of vocabulary entries for pattern matching
#[derive(Debug)]
pub struct Vocabulary {
    pub entries: Vec<VocabEntry>,
}

impl Vocabulary {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn add_entry(&mut self, entry: VocabEntry) {
        self.entries.push(entry);
    }

    /// Load built-in vocabulary entries
    pub fn with_builtins() -> Self {
        let mut vocab = Self::new();

        // compute A + B -> MOVI R0, A; MOVI R1, B; IADD R0, R1; HALT
        vocab.add_entry(VocabEntry::new(
            r#"compute\s+(\d+)\s*\+\s*(\d+)"#,
            "MOVI R0, {0}\nMOVI R1, {1}\nIADD R0, R1\nHALT",
            0,
            "addition"
        ));

        // compute A * B -> MOVI R0, A; MOVI R1, B; IMUL R0, R1; HALT
        vocab.add_entry(VocabEntry::new(
            r#"compute\s+(\d+)\s*\*\s*(\d+)"#,
            "MOVI R0, {0}\nMOVI R1, {1}\nIMUL R0, R1\nHALT",
            0,
            "multiplication"
        ));

        // factorial of N -> loop with IMUL + DEC + JNZ
        vocab.add_entry(VocabEntry::new(
            r#"factorial\s+of\s+(\d+)"#,
            "MOVI R0, 1\nMOVI R1, {0}\nloop:\nCMP R1, 0\nJZ R1, end\nIMUL R0, R1\nDEC R1\nJMP loop\nend:\nHALT",
            0,
            "factorial"
        ));

        // hello -> MOVI R0, 42; HALT
        vocab.add_entry(VocabEntry::new(
            r#"hello"#,
            "MOVI R0, 42\nHALT",
            0,
            "hello"
        ));

        vocab
    }

    /// Find the first vocabulary entry that matches the input
    pub fn match_input(&self, input: &str) -> Option<&VocabEntry> {
        self.entries.iter().find(|entry| {
            let re = match Regex::new(&entry.pattern) {
                Ok(r) => r,
                Err(_) => return false,
            };
            re.is_match(input)
        })
    }
}

impl Default for Vocabulary {
    fn default() -> Self {
        Self::with_builtins()
    }
}

/// Natural language interpreter using vocabulary pattern matching
pub struct Interpreter {
    vocabulary: Vocabulary,
}

impl Interpreter {
    pub fn new(vocabulary: Vocabulary) -> Self {
        Self { vocabulary }
    }

    pub fn with_builtins() -> Self {
        Self::new(Vocabulary::with_builtins())
    }

    /// Execute natural language input and return the result register value
    pub fn execute(&self, input: &str) -> Result<i64, String> {
        // Find matching vocabulary entry
        let entry = self.vocabulary.match_input(input)
            .ok_or_else(|| format!("No pattern matches input: {}", input))?;

        // Substitute captured groups into assembly template
        let assembly = entry.match_and_substitute(input)
            .ok_or_else(|| format!("Pattern match failed for: {}", input))?;

        // Assemble to bytecode
        let bytecode = Assembler::assemble(&assembly)?;

        // Execute in VM
        let mut vm = VmInterpreter::new(&bytecode);
        vm.execute().map_err(|e| format!("VM execution error: {}", e))?;

        // Return result register value
        Ok(vm.read_gp(entry.result_reg) as i64)
    }

    /// Get the vocabulary for inspection/modification
    pub fn vocabulary(&self) -> &Vocabulary {
        &self.vocabulary
    }

    /// Get mutable reference to vocabulary
    pub fn vocabulary_mut(&mut self) -> &mut Vocabulary {
        &mut self.vocabulary
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::with_builtins()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vocab_entry_creation() {
        let entry = VocabEntry::new(
            r#"test\s+(\d+)"#,
            "MOVI R0, {0}",
            0,
            "test"
        );
        assert_eq!(entry.name, "test");
        assert_eq!(entry.result_reg, 0);
    }

    #[test]
    fn test_vocab_entry_match_and_substitute() {
        let entry = VocabEntry::new(
            r#"compute\s+(\d+)\s*\+\s*(\d+)"#,
            "MOVI R0, {0}\nMOVI R1, {1}",
            0,
            "addition"
        );

        let result = entry.match_and_substitute("compute 5 + 3");
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "MOVI R0, 5\nMOVI R1, 3");
    }

    #[test]
    fn test_vocabulary_new() {
        let vocab = Vocabulary::new();
        assert_eq!(vocab.entries.len(), 0);
    }

    #[test]
    fn test_vocabulary_with_builtins() {
        let vocab = Vocabulary::with_builtins();
        assert!(vocab.entries.len() >= 4);
    }

    #[test]
    fn test_vocabulary_add_entry() {
        let mut vocab = Vocabulary::new();
        vocab.add_entry(VocabEntry::new("test", "TEST", 0, "test"));
        assert_eq!(vocab.entries.len(), 1);
    }

    #[test]
    fn test_vocabulary_match_input() {
        let vocab = Vocabulary::with_builtins();

        let matched = vocab.match_input("compute 5 + 3");
        assert!(matched.is_some());
        assert_eq!(matched.unwrap().name, "addition");

        let no_match = vocab.match_input("unknown command");
        assert!(no_match.is_none());
    }

    #[test]
    fn test_interpreter_creation() {
        let interp = Interpreter::with_builtins();
        assert_eq!(interp.vocabulary().entries.len(), 4);
    }

    #[test]
    fn test_interpreter_execute_addition() {
        let interp = Interpreter::with_builtins();
        let result = interp.execute("compute 5 + 3");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 8);
    }

    #[test]
    fn test_interpreter_execute_multiplication() {
        let interp = Interpreter::with_builtins();
        let result = interp.execute("compute 6 * 7");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_interpreter_execute_hello() {
        let interp = Interpreter::with_builtins();
        let result = interp.execute("hello");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_interpreter_execute_factorial() {
        let interp = Interpreter::with_builtins();
        let result = interp.execute("factorial of 5");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 120);
    }

    #[test]
    fn test_interpreter_execute_unknown() {
        let interp = Interpreter::with_builtins();
        let result = interp.execute("unknown command");
        assert!(result.is_err());
    }

    #[test]
    fn test_interpreter_vocabulary_mut() {
        let mut interp = Interpreter::with_builtins();
        interp.vocabulary_mut().add_entry(VocabEntry::new(
            r#"custom\s+(\d+)"#,
            "MOVI R0, {0}\nHALT",
            0,
            "custom"
        ));

        let result = interp.execute("custom 123");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 123);
    }
}
