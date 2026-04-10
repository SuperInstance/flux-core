use flux_core::vocabulary::{VocabEntry, Vocabulary, Interpreter};

#[test]
fn test_vocab_entry_basic() {
    let entry = VocabEntry::new(
        r#"test\s+(\d+)"#,
        "MOVI R0, {0}",
        0,
        "test_entry"
    );

    assert_eq!(entry.name, "test_entry");
    assert_eq!(entry.result_reg, 0);
    assert_eq!(entry.pattern, r#"test\s+(\d+)"#);
    assert_eq!(entry.assembly_template, "MOVI R0, {0}");
}

#[test]
fn test_vocab_entry_single_capture() {
    let entry = VocabEntry::new(
        r#"number\s+(\d+)"#,
        "MOVI R0, {0}\nHALT",
        0,
        "number"
    );

    let result = entry.match_and_substitute("number 42");
    assert!(result.is_some());
    assert_eq!(result.unwrap(), "MOVI R0, 42\nHALT");
}

#[test]
fn test_vocab_entry_multiple_captures() {
    let entry = VocabEntry::new(
        r#"add\s+(\d+)\s+and\s+(\d+)"#,
        "MOVI R0, {0}\nMOVI R1, {1}",
        0,
        "add"
    );

    let result = entry.match_and_substitute("add 10 and 20");
    assert!(result.is_some());
    assert_eq!(result.unwrap(), "MOVI R0, 10\nMOVI R1, 20");
}

#[test]
fn test_vocab_entry_no_match() {
    let entry = VocabEntry::new(
        r#"compute\s+(\d+)"#,
        "MOVI R0, {0}",
        0,
        "compute"
    );

    let result = entry.match_and_substitute("calculate 42");
    assert!(result.is_none());
}

#[test]
fn test_vocabulary_empty() {
    let vocab = Vocabulary::new();
    assert_eq!(vocab.entries.len(), 0);
}

#[test]
fn test_vocabulary_add_single_entry() {
    let mut vocab = Vocabulary::new();
    vocab.add_entry(VocabEntry::new(
        "test",
        "TEST",
        0,
        "test"
    ));

    assert_eq!(vocab.entries.len(), 1);
    assert_eq!(vocab.entries[0].name, "test");
}

#[test]
fn test_vocabulary_builtins_count() {
    let vocab = Vocabulary::with_builtins();
    assert!(vocab.entries.len() >= 4);
}

#[test]
fn test_vocabulary_match_addition() {
    let vocab = Vocabulary::with_builtins();
    let entry = vocab.match_input("compute 5 + 3");

    assert!(entry.is_some());
    assert_eq!(entry.unwrap().name, "addition");
}

#[test]
fn test_vocabulary_match_multiplication() {
    let vocab = Vocabulary::with_builtins();
    let entry = vocab.match_input("compute 6 * 7");

    assert!(entry.is_some());
    assert_eq!(entry.unwrap().name, "multiplication");
}

#[test]
fn test_vocabulary_match_factorial() {
    let vocab = Vocabulary::with_builtins();
    let entry = vocab.match_input("factorial of 5");

    assert!(entry.is_some());
    assert_eq!(entry.unwrap().name, "factorial");
}

#[test]
fn test_vocabulary_match_hello() {
    let vocab = Vocabulary::with_builtins();
    let entry = vocab.match_input("hello");

    assert!(entry.is_some());
    assert_eq!(entry.unwrap().name, "hello");
}

#[test]
fn test_vocabulary_no_match() {
    let vocab = Vocabulary::with_builtins();
    let entry = vocab.match_input("unknown command");

    assert!(entry.is_none());
}

#[test]
fn test_interpreter_default() {
    let interp = Interpreter::default();
    assert!(!interp.vocabulary().entries.is_empty());
}

#[test]
fn test_interpreter_with_builtins() {
    let interp = Interpreter::with_builtins();
    assert!(interp.vocabulary().entries.len() >= 4);
}

#[test]
fn test_interpreter_execute_addition_simple() {
    let interp = Interpreter::with_builtins();
    let result = interp.execute("compute 2 + 3");

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 5);
}

#[test]
fn test_interpreter_execute_addition_larger() {
    let interp = Interpreter::with_builtins();
    let result = interp.execute("compute 100 + 250");

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 350);
}

#[test]
fn test_interpreter_execute_multiplication() {
    let interp = Interpreter::with_builtins();
    let result = interp.execute("compute 12 * 12");

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 144);
}

#[test]
fn test_interpreter_execute_multiplication_zero() {
    let interp = Interpreter::with_builtins();
    let result = interp.execute("compute 0 * 100");

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0);
}

#[test]
fn test_interpreter_execute_hello() {
    let interp = Interpreter::with_builtins();
    let result = interp.execute("hello");

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
}

#[test]
fn test_interpreter_execute_factorial_zero() {
    let interp = Interpreter::with_builtins();
    let result = interp.execute("factorial of 0");

    assert!(result.is_ok());
    // Factorial of 0 is 1 (our loop starts with R0=1 and multiplies until R1=0)
    assert_eq!(result.unwrap(), 1);
}

#[test]
fn test_interpreter_execute_factorial_one() {
    let interp = Interpreter::with_builtins();
    let result = interp.execute("factorial of 1");

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 1);
}

#[test]
fn test_interpreter_execute_factorial_five() {
    let interp = Interpreter::with_builtins();
    let result = interp.execute("factorial of 5");

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 120);
}

#[test]
fn test_interpreter_execute_factorial_six() {
    let interp = Interpreter::with_builtins();
    let result = interp.execute("factorial of 6");

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 720);
}

#[test]
fn test_interpreter_execute_unknown_command() {
    let interp = Interpreter::with_builtins();
    let result = interp.execute("this is not a valid command");

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("No pattern matches"));
}

#[test]
fn test_interpreter_custom_vocabulary() {
    let mut vocab = Vocabulary::new();
    vocab.add_entry(VocabEntry::new(
        r#"square\s+(\d+)"#,
        "MOVI R0, {0}\nMOVI R1, {0}\nIMUL R0, R0, R1\nHALT",
        0,
        "square"
    ));

    let interp = Interpreter::new(vocab);
    let result = interp.execute("square 7");

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 49);
}

#[test]
fn test_interpreter_vocabulary_mutation() {
    let mut interp = Interpreter::with_builtins();
    interp.vocabulary_mut().add_entry(VocabEntry::new(
        r#"double\s+(\d+)"#,
        "MOVI R0, {0}\nMOVI R1, 2\nIMUL R0, R1\nHALT",
        0,
        "double"
    ));

    let result = interp.execute("double 15");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 30);
}

#[test]
fn test_interpreter_whitespace_variations() {
    let interp = Interpreter::with_builtins();

    // Test different whitespace patterns
    assert!(interp.execute("compute 1+2").is_ok());
    assert!(interp.execute("compute 1 +2").is_ok());
    assert!(interp.execute("compute 1+ 2").is_ok());
    assert!(interp.execute("compute  1  +  2  ").is_ok());
}

#[test]
fn test_vocab_entry_substitution_preserves_template() {
    let entry = VocabEntry::new(
        r#"test\s+(.*)"#,
        "MOVI R0, 0\n; {0}\nHALT",
        0,
        "comment"
    );

    let result = entry.match_and_substitute("test anything here");
    assert!(result.is_some());
    let output = result.unwrap();
    assert!(output.contains("MOVI R0, 0"));
    assert!(output.contains("; anything here"));
    assert!(output.contains("HALT"));
}
