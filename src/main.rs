// ==========================================================
// INTELLIGENT-COMPILER FULLSTACK AI EDITION (ONE FILE)
// AI TRANSPILER + FILE CONVERTER + PROJECT-WIDE CONVERTER
// Windows auto-pause + Panic Catcher (절대 창 안 닫힘)
// ==========================================================

use std::collections::HashMap;
use std::env;
use std::fs;
use std::io;
use std::path::Path;
use serde_json::json;

// ----------------------------------------------------------
// PANIC CATCHER (윈도우에서 절대 창 안 닫히게 하는 핵심)
// ----------------------------------------------------------
fn install_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        println!("\n======================================");
        println!("         PANIC OCCURRED");
        println!("======================================");
        println!("{:?}", info);

        // Windows 강제 pause
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
// LLM INTERFACE
// ==========================================================
pub trait LLM {
    fn predict(&self, prompt: &str) -> String;
}

// ==========================================================
// REAL LLM (OPENAI GPT) WITH AUTO API KEY DETECTION
// ==========================================================
#[derive(Clone)]
pub struct RealLLM {
    pub api_key: String,
}

impl RealLLM {
    pub fn new() -> Self {
        if let Ok(v) = env::var("OPENAI_API_KEY") {
            if !v.is_empty() { return Self { api_key: v }; }
        }

        if let Ok(content) = fs::read_to_string(".env") {
            for line in content.lines() {
                if line.starts_with("OPENAI_API_KEY=") {
                    let key = line.split('=').nth(1).unwrap_or("").to_string();
                    if !key.is_empty() {
                        return Self { api_key: key };
                    }
                }
            }
        }

        Self { api_key: "".into() }
    }

    fn request(&self, prompt: &str) -> String {
        if self.api_key.is_empty() {
            return "(ERROR: OPENAI_API_KEY missing. Create .env or export.)".into();
        }

        let client = reqwest::blocking::Client::new();

        let body = json!({
            "model": "gpt-4.1",
            "messages": [{ "role": "user", "content": prompt }]
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
        m.insert("go".into(), vec!["1.18", "1.20", "1.21"]);
        m.insert("cpp".into(), vec!["17", "20", "23"]);
        m.insert("swift".into(), vec!["5.9", "6.0"]);
        Self { map: m }
    }

    pub fn infer(&self, lang: &str, node: &Node) -> String {
        if lang == "go" && node.meta.get("uses_generics") == Some(&"true".into()) {
            return "1.21".into();
        }

        self.map.get(lang)
            .and_then(|v| v.last())
            .unwrap_or(&"unknown")
            .to_string()
    }
}

// ==========================================================
// SEMANTIC ENGINE
// ==========================================================
pub struct SemanticInfo {
    pub meaning: String,
}

pub struct SemanticEngine;

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
pub struct SecurityRule {
    pub name: &'static str,
    pub detect: fn(&Node) -> bool,
}

pub struct SecurityAI<L: LLM> {
    pub llm: L,
    pub rules: Vec<SecurityRule>,
}

impl<L: LLM> SecurityAI<L> {
    pub fn new(llm: L) -> Self {
        Self {
            llm,
            rules: vec![
                SecurityRule {
                    name: "POINTER_ARITH",
                    detect: |n| n.meta.get("pointer_arith") == Some(&"true".into()),
                }
            ]
        }
    }

    pub fn analyze(&self, node: &Node) -> Vec<String> {
        let mut out = vec![];

        for r in &self.rules {
            if (r.detect)(node) {
                out.push(r.name.into());
            }
        }

        out.push(self.llm.predict(&format!("Security check: {:?}", node)));

        out
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
                "swift" => format!("var {}: Any", x),
                "cpp" => format!("auto {};", x),
                "rust" => format!("let {};", x),
                _ => x.into(),
            },

            _ => "/* unsupported */".into(),
        }
    }
}

// ==========================================================
// LLM REFINER
// ==========================================================
pub struct LLMGenerator<L: LLM> {
    pub llm: L,
}

impl<L: LLM> LLMGenerator<L> {
    pub fn refine(&self, lang: &str, version: &str, code: &str) -> String {
        self.llm.predict(
            &format!("Rewrite as valid, idiomatic {} {} code:\n{}", lang, version, code)
        )
    }
}

// ==========================================================
// FILE TRANSPILER
// ==========================================================
pub fn transpile_source<L: LLM>(llm: &L, src: &str, lang: &str) -> String {
    llm.predict(&format!("Transpile entire source to {}:\n{}", lang, src))
}

// ==========================================================
// DIRECTORY TRANSPILER
// ==========================================================
pub fn transpile_directory<L: LLM>(
    llm: &L,
    src_dir: &str,
    out_dir: &str,
    lang: &str
) {
    fs::create_dir_all(out_dir).unwrap_or(());

    fn walk<L: LLM>(llm: &L, src: &Path, out: &Path, lang: &str) {
        for entry in fs::read_dir(src).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();

            if path.is_dir() {
                let next = out.join(entry.file_name());
                fs::create_dir_all(&next).unwrap_or(());
                walk(llm, &path, &next, lang);
                continue;
            }

            if path.is_file() {
                let content = fs::read_to_string(&path).unwrap_or_default();

                let code = llm.predict(
                    &format!("Transpile to {}:\n{}", lang, content)
                );

                let ext = match lang {
                    "go" => "go",
                    "cpp" => "cpp",
                    "swift" => "swift",
                    "python" => "py",
                    "rust" => "rs",
                    _ => "txt",
                };

                let out_path = out.join(format!(
                    "{}.{}",
                    path.file_name().unwrap().to_string_lossy(),
                    ext
                ));

                fs::write(out_path, code).unwrap_or(());
            }
        }
    }

    walk(llm, Path::new(src_dir), Path::new(out_dir), lang);
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
// MAIN (절대 닫히지 않는 버전)
// ==========================================================
fn main() {
    install_panic_hook(); // <<< 윈도우 자동 닫힘 방지의 핵심

    println!("==============================================");
    println!("        INTELLIGENT COMPILER AI ENGINE");
    println!("==============================================");

    let llm = RealLLM::new();
    let compiler = Compiler::new(llm.clone());

    // Node test
    let mut node = Node::new(NodeKind::Identifier("x".into()));
    node.meta.insert("uses_generics".into(), "true".into());
    println!("{}", compiler.compile_node(&node, "go"));

    // File transpile test
    println!("\n=== FILE TRANSPILER ===");
    let sample = r#"
    fn add(a: i32, b: i32) -> i32 { a + b }
    "#;
    println!("{}", transpile_source(&llm, sample, "go"));

    // Directory transpile test
    println!("\n=== PROJECT TRANSPILER START ===");
    transpile_directory(&llm, "src", "output_go", "go");
    println!("=== PROJECT TRANSPILER DONE ===");

    println!("\nFINISHED.");

    // ------------- STOP WINDOW AUTO CLOSE -------------
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

