// ==========================================================
// INTELLIGENT-COMPILER FULLSTACK AI EDITION (ONE FILE)
// WITH AUTO API KEY SETUP + ADVANCED PROJECT TRANSPILER
// ==========================================================

use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use serde_json::json;

// ----------------------------------------------------------
// 1) PANIC CATCHER (창 자동 닫힘 방지)
// ----------------------------------------------------------
fn install_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        println!("\n======================================");
        println!("           PANIC OCCURRED");
        println!("======================================");
        println!("{:?}", info);

        #[cfg(target_os = "windows")]
        {
            println!("\nPress any key to exit...");
            let _ = std::process::Command::new("cmd")
                .args(&["/C", "pause"])
                .status();
        }

        #[cfg(not(target_os = "windows"))]
        {
            println!("\nPress ENTER to exit...");
            let mut s = String::new();
            let _ = io::stdin().read_line(&mut s);
        }
    }));
}

// ==========================================================
// 2) AUTO LOAD OR CREATE API KEY
// ==========================================================
fn load_or_create_api_key() -> String {
    // 1) ENV
    if let Ok(k) = env::var("OPENAI_API_KEY") {
        if !k.trim().is_empty() {
            return k;
        }
    }

    // 2) .env
    if let Ok(content) = fs::read_to_string(".env") {
        for line in content.lines() {
            if line.starts_with("OPENAI_API_KEY=") {
                let key = line.replace("OPENAI_API_KEY=", "");
                if !key.trim().is_empty() {
                    return key.trim().into();
                }
            }
        }
    }

    // 3) 없으면 사용자 입력
    println!("=================================================");
    println!(" OPENAI_API_KEY not found.");
    println!(" Please enter your OpenAI API Key:");
    println!("=================================================");

    print!("API KEY > ");
    io::stdout().flush().unwrap();

    let mut key = String::new();
    io::stdin().read_line(&mut key).unwrap();
    let key = key.trim().to_string();

    if key.is_empty() {
        println!("ERROR: API KEY cannot be empty. Exiting...");
        return "".into();
    }

    // 저장
    let env_file = format!("OPENAI_API_KEY={}", key);
    fs::write(".env", env_file).unwrap();

    println!("API KEY saved to .env.");
    key
}

// ==========================================================
// AST STRUCTURES
// ==========================================================
#[derive(Debug, Clone)]
pub enum NodeKind {
    Identifier(String),
    Number(f64),
    BinaryOp { op: String, left: Box<Node>, right: Box<Node> },
    Function { name: String, args: Vec<String>, body: Vec<Node> },
    Unknown,
}

#[derive(Debug, Clone)]
pub struct Node {
    pub kind: NodeKind,
    pub meta: HashMap<String, String>,
}

impl Node {
    pub fn new(kind: NodeKind) -> Self {
        Self { kind, meta: HashMap::new() }
    }
}

// ==========================================================
// LLM BASE TRAIT
// ==========================================================
pub trait LLM {
    fn predict(&self, prompt: &str) -> String;
}

// ==========================================================
// REAL OPENAI CLIENT
// ==========================================================
#[derive(Clone)]
pub struct RealLLM {
    pub api_key: String,
}

impl RealLLM {
    pub fn new() -> Self {
        // AUTO API KEY SYSTEM 사용
        let key = load_or_create_api_key();
        Self { api_key: key }
    }

    fn request(&self, prompt: &str) -> String {
        if self.api_key.is_empty() {
            return "(ERROR: OPENAI_API_KEY missing.)".into();
        }

        let client = reqwest::blocking::Client::new();

        let body = json!({
            "model": "gpt-4.1",
            "messages": [
                { "role": "user", "content": prompt }
            ]
        });

        let res = client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send();

        match res {
            Ok(r) => {
                let v: serde_json::Value = r.json().unwrap_or(json!({}));
                v["choices"][0]["message"]["content"]
                    .as_str()
                    .unwrap_or("(EMPTY)")
                    .to_string()
            }
            Err(e) => format!("(API ERROR: {})", e),
        }
    }
}

impl LLM for RealLLM {
    fn predict(&self, prompt: &str) -> String {
        self.request(prompt)
    }
}

// ==========================================================
// VERSION AI
// ==========================================================
pub struct VersionAI {
    map: HashMap<String, Vec<&'static str>>,
}

impl VersionAI {
    pub fn new() -> Self {
        let mut m = HashMap::new();
        m.insert("go".into(), vec!["1.21"]);
        m.insert("cpp".into(), vec!["23"]);
        m.insert("swift".into(), vec!["6.0"]);
        Self { map: m }
    }

    pub fn infer(&self, lang: &str, node: &Node) -> String {
        if lang == "go" {
            if node.meta.get("uses_generics") == Some(&"true".to_string()) {
                return "1.21".into();
            }
        }
        self.map
            .get(lang)
            .and_then(|v| v.last())
            .unwrap_or(&"unknown")
            .to_string()
    }
}

// ==========================================================
// SEMANTIC ENGINE
// ==========================================================
pub struct SemanticEngine;

pub struct SemanticInfo {
    pub meaning: String,
}

impl SemanticEngine {
    pub fn analyze(&self, node: &Node) -> SemanticInfo {
        match &node.kind {
            NodeKind::Identifier(x) => SemanticInfo {
                meaning: format!("identifier '{}'", x),
            },
            _ => SemanticInfo { meaning: "unknown".into() }
        }
    }
}

// ==========================================================
// SECURITY AI
// ==========================================================
pub struct SecurityAI<L: LLM> {
    pub llm: L,
}

impl<L: LLM> SecurityAI<L> {
    pub fn new(llm: L) -> Self { Self { llm } }

    pub fn analyze(&self, node: &Node) -> Vec<String> {
        vec![
            self.llm.predict(&format!("Security check for node: {:?}", node))
        ]
    }
}

// ==========================================================
// BASE CODE GENERATOR
// ==========================================================
pub struct BaseGenerator;

impl BaseGenerator {
    pub fn generate(&self, node: &Node, lang: &str) -> String {
        match &node.kind {
            NodeKind::Identifier(x) => match lang {
                "go" => format!("var {} any", x),
                "cpp" => format!("auto {};", x),
                "swift" => format!("var {}: Any", x),
                _ => format!("{}", x),
            },
            _ => "/* unsupported */".into(),
        }
    }
}

// ==========================================================
// LLM REFINER
// ==========================================================
pub struct LLMGenerator<L: LLM> { pub llm: L }

impl<L: LLM> LLMGenerator<L> {
    pub fn refine(&self, lang: &str, version: &str, code: &str) -> String {
        self.llm.predict(
            &format!("Rewrite in idiomatic {} {} code:\n{}", lang, version, code)
        )
    }
}

// ==========================================================
// FILE TRANSPILER
// ==========================================================
pub fn transpile_file<L: LLM>(llm: &L, src: &str, lang: &str) -> String {
    llm.predict(&format!("Transpile to {}:\n{}", lang, src))
}

// ==========================================================
// 5) ADVANCED PROJECT DIRECTORY TRANSPILER
// ==========================================================

// 기본적으로 변환하지 않을 폴더
fn should_skip_dir(path: &Path) -> bool {
    let skip_list = [
        ".git",
        "target",
        "build",
        "node_modules",
        "__pycache__",
        ".idea",
        ".vscode",
    ];

    skip_list.iter().any(|&name| path.ends_with(name))
}

// 변환할 파일 확장자
fn is_convertible_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        let e = ext.to_string_lossy().to_lowercase();
        return matches!(e.as_str(), "rs" | "cpp" | "h" | "c" | "py" | "go" | "ts" | "js" | "swift");
    }
    false
}

// 언어별 변환된 확장자
fn mapped_ext(lang: &str) -> &'static str {
    match lang {
        "go" => "go",
        "cpp" => "cpp",
        "swift" => "swift",
        "rust" => "rs",
        "python" => "py",
        _ => "txt",
    }
}

pub fn transpile_project<L: LLM>(
    llm: &L,
    src_dir: &str,
    out_dir: &str,
    lang: &str,
) {
    println!("\n--- PROJECT TRANSPILER START ---");
    fs::create_dir_all(out_dir).unwrap();

    fn walk<L: LLM>(llm: &L, src: &Path, out: &Path, lang: &str) {
        for entry in fs::read_dir(src).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();

            if path.is_dir() {
                if should_skip_dir(&path) {
                    println!("[SKIP] directory: {}", path.display());
                    continue;
                }

                let next = out.join(entry.file_name());
                fs::create_dir_all(&next).unwrap_or(());
                walk(llm, &path, &next, lang);
            } else if path.is_file() {
                if !is_convertible_file(&path) {
                    println!("[IGNORE] {}", path.display());
                    continue;
                }

                println!("[CONVERT] {}", path.display());
                let content = fs::read_to_string(&path).unwrap_or_default();

                let code = llm.predict(
                    &format!("Transpile fully into {} code:\n{}", lang, content)
                );

                let newname = format!(
                    "{}.{}",
                    path.file_name().unwrap().to_string_lossy(),
                    mapped_ext(lang)
                );
                fs::write(out.join(newname), code).unwrap_or(());
            }
        }
    }

    walk(llm, Path::new(src_dir), Path::new(out_dir), lang);
    println!("--- PROJECT TRANSPILER DONE ---");
}

// ==========================================================
// FULL INTELLIGENT COMPILER
// ==========================================================
pub struct Compiler<L: LLM + Clone> {
    pub llm: L,
    pub version_ai: VersionAI,
    pub semantic: SemanticEngine,
    pub security: SecurityAI<L>,
}

impl<L: LLM + Clone> Compiler<L> {
    pub fn new(llm: L) -> Self {
        Self {
            version_ai: VersionAI::new(),
            semantic: SemanticEngine,
            security: SecurityAI::new(llm.clone()),
            llm
        }
    }

    pub fn compile_node(&self, node: &Node, lang: &str) -> String {
        let ver = self.version_ai.infer(lang, node);
        let sem = self.semantic.analyze(node);
        let base = BaseGenerator.generate(node, lang);
        let refined = LLMGenerator { llm: self.llm.clone() }.refine(lang, &ver, &base);
        let sec = self.security.analyze(node);

        format!(
            "=== Intelligent Compiler ===\n\
             Language: {}\nVersion: {}\nMeaning: {}\n\n\
             Base:\n{}\n\nAI Refined:\n{}\n\nSecurity:\n{:?}",
            lang, ver, sem.meaning, base, refined, sec
        )
    }
}

// ==========================================================
// MAIN
// ==========================================================
fn main() {
    install_panic_hook();

    println!("==============================================");
    println!("        INTELLIGENT COMPILER AI ENGINE");
    println!("==============================================");

    let llm = RealLLM::new();
    let compiler = Compiler::new(llm.clone());

    // Test Node
    let mut node = Node::new(NodeKind::Identifier("x".into()));
    node.meta.insert("uses_generics".into(), "true".into());
    println!("{}", compiler.compile_node(&node, "go"));

    // Test File
    println!("\n=== FILE TRANSPILER ===");
    let sample = "fn add(a: i32, b: i32) -> i32 { a + b }";
    println!("{}", transpile_file(&llm, sample, "go"));

    // Test Project
    println!("\n=== PROJECT TRANSPILER ===");
    transpile_project(&llm, "src", "output_go", "go");

    println!("\nFINISHED.");

    #[cfg(target_os = "windows")]
    {
        println!("Press any key to exit...");
        let _ = std::process::Command::new("cmd")
            .args(&["/C", "pause"])
            .status();
    }

    #[cfg(not(target_os = "windows"))]
    {
        println!("Press ENTER to exit...");
        let mut s = String::new();
        let _ = io::stdin().read_line(&mut s);
    }
}
