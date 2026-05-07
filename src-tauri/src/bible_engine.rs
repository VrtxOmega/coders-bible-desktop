/// The Coder's Bible — Rust Knowledge Engine
/// Ported from Python bible_engine.py with deterministic fidelity.
/// Zero AI. Zero network. Pure signal.

use regex::Regex;
use rusqlite::{Connection, Result as SqlResult};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

// ─── Data Structures ───────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct DetectionResult {
    pub language: String,
    pub confidence: f64,
    pub confidence_label: String,
    pub reasoning: String,
    pub context: String,
    pub color: String,
    pub alternatives: Vec<AltDetection>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AltDetection {
    pub language: String,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SafetyResult {
    pub level: String,
    pub warnings: Vec<SafetyWarning>,
    pub safe_notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SafetyWarning {
    pub level: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FragmentResult {
    pub id: String,
    pub content: String,
    pub source: String,
    pub tier: String,
    pub relevance: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchResult {
    pub query: String,
    pub results: Vec<FragmentResult>,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct BreakdownItem {
    pub code: String,
    pub concept: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AnalysisResult {
    pub input: String,
    pub language: DetectionResult,
    pub safety: SafetyResult,
    pub keywords: Vec<String>,
    pub breakdown: Vec<BreakdownItem>,
    pub quick_understanding: String,
    pub results: Vec<FragmentResult>,
    pub result_count: usize,
    pub total_fragments: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DomainStat {
    pub name: String,
    pub count: i64,
    pub color: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct StatsResult {
    pub total_fragments: i64,
    pub domains: Vec<DomainStat>,
}

// ─── Constants ─────────────────────────────────────────────

use once_cell::sync::Lazy;

static DOMAIN_COLORS: Lazy<HashMap<&str, &str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("Python", "#3776AB");
    m.insert("JavaScript", "#F7DF1E");
    m.insert("TypeScript", "#3178C6");
    m.insert("Rust", "#DEA584");
    m.insert("Go", "#00ADD8");
    m.insert("Ruby", "#CC342D");
    m.insert("PHP", "#777BB4");
    m.insert("Java", "#ED8B00");
    m.insert("Bash", "#4EAA25");
    m.insert("PowerShell", "#012456");
    m.insert("SQL", "#E38C00");
    m.insert("Docker", "#2496ED");
    m.insert("Kubernetes", "#326CE5");
    m.insert("Nginx", "#009639");
    m.insert("systemd", "#6C7A89");
    m.insert("Git", "#F05032");
    m.insert("Ansible", "#EE0000");
    m.insert("CSS", "#1572B6");
    m.insert("HTML", "#E34F26");
    m.insert("YAML", "#CB171E");
    m.insert("JSON", "#292929");
    m.insert("Terraform", "#7B42BC");
    m.insert("C#", "#239120");
    m.insert("Kotlin", "#7F52FF");
    m.insert("C/C++", "#00599C");
    m.insert("Swift", "#F05138");
    m.insert("Linux", "#FCC624");
    m.insert("Node.js", "#339933");
    m.insert("MySQL", "#4479A1");
    m.insert("PostgreSQL", "#336791");
    m.insert("Other", "#666666");
    m
});

static LANGUAGE_SOURCE_MAP: Lazy<HashMap<&str, Vec<&str>>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("Python", vec!["%python%", "%docs.python.org%"]);
    m.insert("JavaScript", vec!["%nodejs%", "%npmjs%", "%node.js%"]);
    m.insert("TypeScript", vec!["%typescript%", "%typescriptlang%"]);
    m.insert("Rust", vec!["%rust%", "%doc.rust-lang%", "%docs.rs%"]);
    m.insert("Go", vec!["%golang%", "%go.dev%", "%pkg.go.dev%"]);
    m.insert("Ruby", vec!["%ruby%"]);
    m.insert("PHP", vec!["%php%"]);
    m.insert("Java", vec!["%java%"]);
    m.insert("Bash", vec!["%bash%", "%gnu.org/software/bash%", "%help:%", "%tldp.org%"]);
    m.insert("PowerShell", vec!["%powershell%", "%microsoft.com/powershell%"]);
    m.insert("SQL", vec!["%mysql%", "%mariadb%", "%postgresql%", "%postgres%", "%sqlite%"]);
    m.insert("Docker", vec!["%docker%"]);
    m.insert("Kubernetes", vec!["%kubernetes%", "%k8s%"]);
    m.insert("Nginx", vec!["%nginx%"]);
    m.insert("systemd", vec!["%systemd%", "%freedesktop.org%"]);
    m.insert("Git", vec!["%git-scm%", "%git/%"]);
    m.insert("Ansible", vec!["%ansible%"]);
    m.insert("Terraform", vec!["%terraform%"]);
    m.insert("Linux", vec!["%linux%", "%man7.org%", "%sourceware.org%"]);
    m.insert("CSS", vec!["%css%", "%w3.org%"]);
    m.insert("HTML", vec!["%html%", "%w3.org%"]);
    m
});

static SYNTAX_NOISE: Lazy<HashSet<&str>> = Lazy::new(|| {
    let mut s = HashSet::new();
    // Universal
    for w in ["the","a","an","is","are","was","were","be","been","being","have","has","had","do","does","did","will","would","could","should","may","might","must","shall","not","and","or","but","so","yet","nor","if","else","elif","then","fi","end","done","for","while","in","of","to","from","with","as","true","false","null","none","nil","void","undefined"] {
        s.insert(w);
    }
    // Python
    for w in ["def","class","import","return","pass","break","continue","try","except","finally","raise","with","yield","async","await","lambda","global","nonlocal","assert","del","self","cls","print","len","range","type","list","dict","set","tuple","int","str","bool","float","bytes"] {
        s.insert(w);
    }
    // JS/TS
    for w in ["var","let","const","function","new","this","super","typeof","instanceof","switch","case","default","throw","catch","export","extends","implements","interface","enum","abstract","static","readonly","public","private","protected","string","number","boolean","any","never","unknown","object","symbol","console","log","module"] {
        s.insert(w);
    }
    // Rust
    for w in ["fn","pub","mod","use","crate","extern","impl","trait","struct","match","mut","ref","move","unsafe","where","loop","Some","None","Ok","Err","unwrap","expect"] {
        s.insert(w);
    }
    // Go
    for w in ["func","package","chan","select","defer","go","map","make","append","fmt","err","error","nil"] {
        s.insert(w);
    }
    // Ruby
    for w in ["puts","require","attr","begin","rescue","ensure","unless","until","when","defined"] {
        s.insert(w);
    }
    // PHP
    for w in ["echo","isset","empty","unset","array","foreach","include","namespace","php"] {
        s.insert(w);
    }
    // Java
    for w in ["void","main","args","System","out","println","extends","implements","throws","final","native"] {
        s.insert(w);
    }
    // Shell
    for w in ["fi","esac","done","do","then","elif","exit","shift","source","eval","exec"] {
        s.insert(w);
    }
    // SQL
    for w in ["select","insert","update","delete","create","alter","drop","table","index","view","where","join","left","right","inner","outer","group","order","having","limit","offset","values","into","column"] {
        s.insert(w);
    }
    // Docker
    for w in ["run","cmd","copy","add","expose","env","arg","workdir","volume","user","label","entrypoint"] {
        s.insert(w);
    }
    // K8s
    for w in ["kind","spec","metadata","name","containers","image","ports","replicas"] {
        s.insert(w);
    }
    // Nginx
    for w in ["server","location","upstream","listen","root","proxy","http","events","worker"] {
        s.insert(w);
    }
    // Git
    for w in ["git","commit","push","pull","fetch","merge","rebase","checkout","branch","remote","origin"] {
        s.insert(w);
    }
    // Ansible
    for w in ["hosts","tasks","roles","vars","handlers","become","playbook","inventory"] {
        s.insert(w);
    }
    // Terraform
    for w in ["resource","variable","output","provider","data","module","locals","terraform"] {
        s.insert(w);
    }
    // CSS/HTML
    for w in ["div","span","body","head","html","style","script","color","background","margin","padding","display","flex","grid","position","border","font"] {
        s.insert(w);
    }
    s
});

static KNOWN_BINARIES: Lazy<HashMap<&str, (&str, &str)>> = Lazy::new(|| {
    let mut m = HashMap::new();
    // Shell / Unix
    m.insert("chmod", ("Bash", "Changes file permissions (read/write/execute) for users, groups, or everyone."));
    m.insert("chown", ("Bash", "Changes the owner and/or group of a file or directory."));
    m.insert("chgrp", ("Bash", "Changes the group ownership of a file or directory."));
    m.insert("mkdir", ("Bash", "Creates a new directory (folder) at the specified path."));
    m.insert("rmdir", ("Bash", "Removes an empty directory."));
    m.insert("cp", ("Bash", "Copies files or directories from one location to another."));
    m.insert("mv", ("Bash", "Moves or renames files and directories."));
    m.insert("rm", ("Bash", "Deletes files or directories. Use with caution."));
    m.insert("ls", ("Bash", "Lists the contents of a directory."));
    m.insert("cat", ("Bash", "Displays the contents of a file."));
    m.insert("grep", ("Bash", "Searches for text patterns inside files."));
    m.insert("awk", ("Bash", "Processes and transforms structured text data."));
    m.insert("sed", ("Bash", "Edits text in a stream or file using pattern matching."));
    m.insert("find", ("Bash", "Searches for files and directories matching specified criteria."));
    m.insert("tar", ("Bash", "Creates or extracts compressed archive files."));
    m.insert("curl", ("Bash", "Transfers data to or from a server using URLs."));
    m.insert("wget", ("Bash", "Downloads files from the internet."));
    m.insert("ssh", ("Bash", "Opens a secure remote shell connection to another machine."));
    m.insert("scp", ("Bash", "Copies files securely between machines over SSH."));
    m.insert("rsync", ("Bash", "Synchronizes files between directories or machines efficiently."));
    m.insert("sudo", ("Bash", "Runs a command with administrator (root) privileges."));
    m.insert("apt", ("Bash", "Installs, updates, or removes packages on Debian/Ubuntu systems."));
    m.insert("yum", ("Bash", "Installs, updates, or removes packages on RHEL/CentOS systems."));
    m.insert("dnf", ("Bash", "Installs, updates, or removes packages on Fedora systems."));
    m.insert("brew", ("Bash", "Installs, updates, or removes packages on macOS."));
    m.insert("pacman", ("Bash", "Installs, updates, or removes packages on Arch Linux."));
    m.insert("systemctl", ("Bash", "Controls system services (start, stop, enable, disable)."));
    m.insert("journalctl", ("Bash", "Views system logs from the journal."));
    m.insert("crontab", ("Bash", "Schedules commands to run automatically at specified times."));
    m.insert("echo", ("Bash", "Prints text to the terminal output."));
    m.insert("export", ("Bash", "Sets an environment variable for the current session."));
    // Git
    m.insert("git", ("Git", "Runs a Git version control operation."));
    // Docker
    m.insert("docker", ("Docker", "Runs a Docker container management command."));
    m.insert("docker-compose", ("Docker", "Manages multi-container Docker applications."));
    // K8s
    m.insert("kubectl", ("Kubernetes", "Sends commands to a Kubernetes cluster."));
    m.insert("helm", ("Kubernetes", "Manages Kubernetes application packages (charts)."));
    // Python
    m.insert("python", ("Python", "Runs a Python script or starts the Python interpreter."));
    m.insert("python3", ("Python", "Runs a Python 3 script or starts the Python 3 interpreter."));
    m.insert("pip", ("Python", "Installs or manages Python packages."));
    m.insert("pip3", ("Python", "Installs or manages Python 3 packages."));
    // Node
    m.insert("node", ("JavaScript", "Runs a JavaScript file in the Node.js runtime."));
    m.insert("npm", ("JavaScript", "Manages Node.js packages and runs scripts."));
    m.insert("npx", ("JavaScript", "Runs a Node.js package without installing it globally."));
    m.insert("yarn", ("JavaScript", "Manages Node.js packages (alternative to npm)."));
    // Rust
    m.insert("cargo", ("Rust", "Builds, tests, or manages Rust projects and dependencies."));
    m.insert("rustc", ("Rust", "Compiles Rust source code into a binary."));
    // Go
    m.insert("go", ("Go", "Runs a Go toolchain command (build, run, test, etc.)."));
    // Terraform
    m.insert("terraform", ("Terraform", "Manages infrastructure as code (plan, apply, destroy)."));
    // Ansible
    m.insert("ansible", ("Ansible", "Runs Ansible automation tasks on remote machines."));
    m.insert("ansible-playbook", ("Ansible", "Executes an Ansible playbook for configuration management."));
    // Nginx
    m.insert("nginx", ("Nginx", "Controls the Nginx web server."));
    m
});

static SAFETY_CONTEXT: Lazy<HashMap<&str, &str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("chmod", "This command modifies file permissions. Safe in most cases, but ensure the file is trusted before executing.");
    m.insert("chown", "This changes file ownership. Requires appropriate privileges and affects who can access the file.");
    m.insert("chgrp", "This changes group ownership of a file. Safe when targeting known files.");
    m.insert("rm", "This deletes files. Double-check the target path before running — deletion is usually permanent.");
    m.insert("mv", "This moves or renames files. The original location will no longer contain the file.");
    m.insert("cp", "This copies files. Safe operation — originals are preserved.");
    m.insert("mkdir", "This creates a new directory. Safe, non-destructive operation.");
    m.insert("git", "This is a Git version control operation. Most Git commands are safe and reversible.");
    m.insert("docker", "This runs a Docker container command. Container changes are isolated from your host system.");
    m.insert("kubectl", "This sends a command to a Kubernetes cluster. Verify you are targeting the correct cluster and namespace.");
    m.insert("pip", "This installs or manages Python packages. Packages are installed into your current environment.");
    m.insert("pip3", "This installs or manages Python packages. Packages are installed into your current environment.");
    m.insert("npm", "This manages Node.js packages. Packages are installed into the project or global directory.");
    m.insert("yarn", "This manages Node.js packages. Packages are installed into the project directory.");
    m.insert("apt", "This installs or modifies system packages. May require sudo and affects system-wide state.");
    m.insert("brew", "This installs or manages macOS packages. Packages install into the Homebrew prefix directory.");
    m.insert("sudo", "This runs the following command with elevated privileges. Ensure you trust the operation before proceeding.");
    m.insert("curl", "This transfers data to/from a URL. Safe for reading; inspect the response before piping to other commands.");
    m.insert("wget", "This downloads a file from the internet. Verify the URL is from a trusted source.");
    m.insert("ssh", "This opens a secure shell connection. Ensure the target host is trusted.");
    m.insert("systemctl", "This controls system services. Starting/stopping services affects system behavior.");
    m.insert("terraform", "This manages cloud infrastructure. Plan first — apply changes can create or destroy real resources.");
    m.insert("ansible", "This runs automation tasks on remote machines. Changes will be applied to the targeted hosts.");
    m.insert("ansible-playbook", "This executes an Ansible playbook. Changes will be applied to the targeted hosts.");
    m.insert("cargo", "This manages a Rust project. Build and test operations are safe; publish is permanent.");
    m.insert("go", "This runs a Go toolchain command. Build and test operations are safe.");
    m.insert("node", "This runs JavaScript in Node.js. The script will have access to your filesystem.");
    m.insert("python", "This runs a Python script. The script will have access to your filesystem.");
    m.insert("python3", "This runs a Python 3 script. The script will have access to your filesystem.");
    m
});

static CONCEPT_PATTERNS: Lazy<Vec<(&str, &str)>> = Lazy::new(|| {
    vec![
        (r"\b(\w+)\s*\(.*\b\1\b", "recursion"),
        (r"\bfor\b.*\bin\b|\bwhile\b", "iteration loop"),
        (r"\bclass\b.*\(.*\)", "inheritance"),
        (r"\btry\b.*\b(except|catch)\b", "error handling exception"),
        (r"\basync\b|\bawait\b", "asynchronous concurrency"),
        (r"\blambda\b|=>", "lambda anonymous function"),
        (r"\bsort\b|\bsorted\b", "sorting algorithm"),
        (r"\b(open|read|write|close)\s*\(", "file io"),
        (r"\b(get|post|put|delete|fetch|request)\b", "http request api"),
        (r"\b(connect|cursor|execute|query)\b", "database query"),
        (r"\b(import|require|use|include)\b.*\b(json|yaml|xml|csv)\b", "serialization parsing"),
        (r"\b(re\.|regex|match|search|findall)\b", "regular expression pattern"),
        (r"\b(thread|process|pool|concurrent|parallel)\b", "concurrency threading"),
        (r"\b(socket|listen|bind|accept)\b", "networking socket"),
        (r"\b(encrypt|decrypt|hash|hmac|sha|md5)\b", "cryptography security"),
        (r"\b(test|assert|expect|mock)\b", "testing unit test"),
    ]
});

static DANGEROUS_PATTERNS: Lazy<Vec<(&str, &str)>> = Lazy::new(|| {
    vec![
        (r"\brm\s+-rf\b", "Recursive forced deletion — can destroy entire directory trees"),
        (r"\b(dd\s+if=|mkfs|fdisk|parted)\b", "Disk-level operations — can wipe drives"),
        (r"\b(chmod\s+777|chmod\s+-R\s+777)\b", "World-writable permissions — security risk"),
        (r"\b(eval|exec)\s*\(", "Dynamic code execution — injection risk if input is untrusted"),
        (r"\b(DROP\s+TABLE|DROP\s+DATABASE|TRUNCATE)\b", "Destructive database operation — data loss"),
        (r"\b(curl|wget).*\|\s*(bash|sh)\b", "Piping remote script to shell — arbitrary code execution risk"),
        (r">\s*/dev/sd[a-z]", "Writing directly to block device — will destroy filesystem"),
        (r"\b(:\(\)\{.*\};)\b", "Fork bomb — will crash the system"),
    ]
});

static SAFE_PATTERNS: Lazy<Vec<(&str, &str)>> = Lazy::new(|| {
    vec![
        (r"^\s*(ls|pwd|whoami|hostname|date|uptime|cat|head|tail|wc|echo)\b", "Read-only / informational command"),
        (r"^\s*(git\s+(status|log|diff|branch))\b", "Read-only Git operation"),
        (r"^\s*(SELECT|SHOW|DESCRIBE|EXPLAIN)\b", "Read-only database query"),
        (r"^\s*(print|console\.log|puts|echo|fmt\.Println)\b", "Output/display operation"),
    ]
});

// ─── BibleEngine ───────────────────────────────────────────

pub struct BibleEngine {
    db_path: String,
}

impl BibleEngine {
    pub fn new(db_path: String) -> Self {
        Self { db_path }
    }

    fn conn(&self) -> SqlResult<Connection> {
        Connection::open(&self.db_path)
    }

    pub fn total_fragments(&self) -> i64 {
        self.conn()
            .and_then(|c| {
                c.query_row("SELECT COUNT(*) FROM fragments", [], |row| row.get(0))
            })
            .unwrap_or(0)
    }

    // ─── Language Detection ────────────────────────────────

    pub fn detect_language(&self, snippet: &str) -> DetectionResult {
        let lines: Vec<&str> = snippet.lines().collect();
        let mut scores: HashMap<String, LangScore> = HashMap::new();

        // High-signal patterns
        let high_signal: HashMap<&str, Vec<&str>> = [
            ("Python", vec![r"^\s*def\s+\w+\s*\(.*\)\s*(->\s*\w+)?\s*:", r"^\s*class\s+\w+(\(.*\))?\s*:", r#"^\s*if\s+__name__\s*==\s*['"]__main__['"]"#,
 r"^\s*(import|from)\s+\w+"]),
            ("JavaScript", vec![r"^\s*function\s+\w+\s*\(", r"^\s*(const|let|var)\s+\w+\s*=", r"\bconsole\.(log|warn|error|info)\s*\(", r"^\s*(module\.exports|export\s+(default|const|function|class))\b"]),
            ("TypeScript", vec![r"^\s*interface\s+\w+", r"^\s*type\s+\w+\s*=" , r":\s*(string|number|boolean|void|any|never|unknown)\b"]),
            ("Rust", vec![r"^\s*fn\s+\w+\s*(<.*>)?\s*\(", r"^\s*(pub|mod|use|impl|trait|struct|enum)\b", r"^\s*#\[(derive|cfg|test)\b"]),
            ("Go", vec![r"^\s*func\s+(\(\w+\s+\*?\w+\)\s+)?\w+\s*\(", r"^\s*package\s+\w+"]),
            ("SQL", vec![r"^\s*(SELECT|INSERT|UPDATE|DELETE|CREATE|ALTER|DROP)\b"]),
            ("Docker", vec![r"^\s*(FROM|RUN|CMD|ENTRYPOINT|COPY|ADD|EXPOSE|ENV|ARG|WORKDIR)\b"]),
            ("Kubernetes", vec![r"^\s*apiVersion:\s*", r"^\s*kind:\s*(Pod|Deployment|Service|ConfigMap|Secret|Ingress)\b"]),
            ("Java", vec![r"^\s*(public|private|protected)\s+(static\s+)?(void|int|String|boolean)\s+\w+\s*\(", r"^\s*import\s+java\."]),
            ("PHP", vec![r"<\?php"]),
            ("Ruby", vec![r"\b(puts|require|require_relative|attr_accessor|attr_reader)\b"]),
        ].into_iter().collect();

        // Full fingerprints (language, patterns, priority)
        let fingerprints: Vec<(&str, Vec<&str>, i32)> = vec![
            ("Bash", vec![
                r"^\s*#!/bin/(ba)?sh",
                r"^\s*(sudo|apt|yum|dnf|brew|pacman|chmod|chown|chgrp|mkdir|rmdir|cp|mv|rm|ls|cat|grep|awk|sed|find|xargs|tar|curl|wget|ssh|scp|rsync|systemctl|journalctl|crontab)\b",
                r"^\s*export\s+\w+=",
                r"\|\s*(grep|awk|sed|sort|uniq|wc|head|tail|cut|tr)\b",
                r"^\s*(if|then|fi|elif|else|for|do|done|while|case|esac)\b.*;\s*$",
                r"\$\{?\w+\}?",
                r"^\s*echo\s+",
            ], 10),
            ("PowerShell", vec![
                r"^\s*(Get-|Set-|New-|Remove-|Invoke-|Start-|Stop-|Write-|Read-|Test-|Import-|Export-|Select-|Where-|ForEach-)\w+",
                r"\$\w+\s*=\s*",
                r"\|\s*(Select-Object|Where-Object|ForEach-Object|Sort-Object|Group-Object|Measure-Object)\b",
                r"^\s*\[Parameter\(",
                r"^\s*param\s*\(",
                r"-\w+\s+\$",
            ], 10),
            ("Python", vec![
                r"^\s*def\s+\w+\s*\(.*\)\s*(->\s*\w+)?\s*:",
                r"^\s*class\s+\w+(\(.*\))?\s*:",
                r"^\s*(import|from)\s+\w+",
                r#"^\s*if\s+__name__\s*==\s*['"]__main__['"]"#,

                r"^\s*@\w+",
                r"^\s*(print|len|range|enumerate|zip|map|filter|lambda)\s*\(",
                r"^\s*(try|except|finally|raise|with|as|yield|async|await)\b",
                r"self\.\w+",
            ], 9),
            ("JavaScript", vec![
                r"^\s*(const|let|var)\s+\w+\s*=",
                r"^\s*function\s+\w+\s*\(",
                r"=>\s*\{",
                r"\bconsole\.(log|warn|error|info)\s*\(",
                r"^\s*(module\.exports|export\s+(default|const|function|class))\b",
                r"\b(document|window|addEventListener|querySelector|fetch)\b",
                r#"^\s*import\s+.*\s+from\s+['\"]"#,
                r"\.then\s*\(|\.catch\s*\(|async\s+function",
            ], 8),
            ("TypeScript", vec![
                r":\s*(string|number|boolean|void|any|never|unknown|undefined)\b",
                r"^\s*interface\s+\w+",
                r"^\s*type\s+\w+\s*=",
                r"<\w+(\s*,\s*\w+)*>",
                r"^\s*(public|private|protected|readonly)\s+",
                r"\bas\s+(string|number|boolean|any|unknown)\b",
                r"^\s*enum\s+\w+\s*\{",
            ], 9),
            ("Rust", vec![
                r"^\s*fn\s+\w+\s*(<.*>)?\s*\(",
                r"^\s*(pub|mod|use|crate|extern|impl|trait|struct|enum)\b",
                r"\b(let\s+mut|&mut|&self|Self|Option<|Result<|Vec<|Box<|Rc<|Arc<)\b",
                r"^\s*#\[(derive|cfg|test|allow|warn|deny)\b",
                r"\b(unwrap|expect|match|Some|None|Ok|Err)\b",
                r"^\s*macro_rules!\s*\w+",
                r"::\s*new\s*\(",
            ], 9),
            ("Go", vec![
                r"^\s*func\s+(\(\w+\s+\*?\w+\)\s+)?\w+\s*\(",
                r"^\s*package\s+\w+",
                r"^\s*import\s+\(",
                r"\b(fmt|os|io|net|http|json|sync|context|errors)\.\w+",
                r":=\s*",
                r"^\s*type\s+\w+\s+(struct|interface)\s*\{",
                r"\bgo\s+func\b|\bdefer\b|\bgoroutine\b",
                r"\bchan\s+\w+|\bselect\s*\{",
            ], 9),
            ("Ruby", vec![
                r"^\s*def\s+\w+",
                r"^\s*class\s+\w+(\s*<\s*\w+)?",
                r"^\s*module\s+\w+",
                r"\b(puts|print|require|require_relative|attr_accessor|attr_reader|attr_writer)\b",
                r"^\s*end\s*$",
                r"\bdo\s*\|.*\|",
                r"\.(each|map|select|reject|reduce|inject|collect|detect|find)\b",
                r#"^\s*gem\s+['\"]"#,
            ], 8),
            ("PHP", vec![
                r"<\?php",
                r"\$\w+\s*=",
                r"^\s*(function|class|interface|trait|namespace|use)\b",
                r"\b(echo|print_r|var_dump|isset|empty|unset|array|foreach|require_once|include)\b",
                r"->\w+\s*\(",
                r"::\w+\s*\(",
            ], 8),
            ("Java", vec![
                r"^\s*(public|private|protected)\s+(static\s+)?(void|int|String|boolean|double|float|long|char)\s+\w+\s*\(",
                r"^\s*class\s+\w+(\s+extends\s+\w+)?(\s+implements\s+\w+)?\s*\{",
                r"^\s*import\s+java\.",
                r"\b(System\.out\.println|System\.err|new\s+\w+\()\b",
                r"^\s*@(Override|Test|Autowired|Component|Service|Repository|Controller)\b",
                r"\b(ArrayList|HashMap|LinkedList|TreeMap|StringBuilder)\b",
            ], 8),
            ("SQL", vec![
                r"^\s*(SELECT|INSERT|UPDATE|DELETE|CREATE|ALTER|DROP|GRANT|REVOKE)\b",
                r"\b(FROM|WHERE|JOIN|LEFT|RIGHT|INNER|OUTER|GROUP\s+BY|ORDER\s+BY|HAVING|LIMIT|OFFSET)\b",
                r"\b(TABLE|INDEX|VIEW|TRIGGER|PROCEDURE|FUNCTION|DATABASE|SCHEMA)\b",
                r"\b(INT|VARCHAR|TEXT|BOOLEAN|TIMESTAMP|SERIAL|PRIMARY\s+KEY|FOREIGN\s+KEY|NOT\s+NULL)\b",
            ], 7),
            ("Docker", vec![
                r"^\s*(FROM|RUN|CMD|ENTRYPOINT|COPY|ADD|EXPOSE|ENV|ARG|WORKDIR|VOLUME|USER|LABEL|HEALTHCHECK)\b",
                r"^\s*docker\s+(build|run|exec|compose|pull|push|images|ps|stop|rm|logs|inspect)\b",
            ], 8),
            ("Kubernetes", vec![
                r"^\s*apiVersion:\s*",
                r"^\s*kind:\s*(Pod|Deployment|Service|ConfigMap|Secret|Ingress|DaemonSet|StatefulSet)\b",
                r"^\s*kubectl\s+(get|apply|describe|delete|logs|exec|port-forward|scale)\b",
                r"^\s*metadata:\s*$",
                r"^\s*spec:\s*$",
            ], 8),
            ("Nginx", vec![
                r"^\s*(server|location|upstream|http|events)\s*\{",
                r"\b(proxy_pass|listen|server_name|root|index|try_files|fastcgi_pass|ssl_certificate)\b",
                r"^\s*worker_processes\b",
                r"^\s*include\s+/etc/nginx/",
            ], 8),
            ("systemd", vec![
                r"^\s*\[(Unit|Service|Install|Timer|Socket|Mount|Path)\]",
                r"^\s*(ExecStart|ExecStop|ExecReload|Restart|WantedBy|After|Before|Requires|Description)\s*=",
                r"^\s*systemctl\s+(start|stop|restart|enable|disable|status|daemon-reload)\b",
            ], 8),
            ("Git", vec![
                r"^\s*git\s+(init|clone|add|commit|push|pull|fetch|merge|rebase|checkout|branch|log|diff|stash|reset|tag|remote)\b",
                r"^\s*\.gitignore\b",
            ], 7),
            ("Ansible", vec![
                r"^\s*-\s*(name|hosts|tasks|roles|vars|handlers|become|gather_facts):",
                r"\b(ansible|playbook|inventory|galaxy|vault)\b",
                r"^\s*\w+\.\w+\.\w+:",
            ], 7),
            ("YAML", vec![
                r"^\s*\w+:\s*$",
                r"^\s*-\s+\w+:",
                r"^\s*---\s*$",
            ], 3),
            ("JSON", vec![
                r#"^\s*\{[\s\n]*\"\w+\""#,
                r"^\s*\[[\s\n]*\{",
            ], 3),
            ("CSS", vec![
                r"^\s*[\.\#\w\[\*:]+\s*\{",
                r"\b(color|background|margin|padding|display|flex|grid|position|border|font|transition|animation|transform)\s*:",
                r"^\s*@(media|keyframes|import|font-face)\b",
            ], 6),
            ("HTML", vec![
                r"^\s*<(html|head|body|div|span|p|a|img|form|input|button|script|style|link|meta)\b",
                r"</\w+>",
            ], 5),
        ];

        for (lang, patterns, priority) in fingerprints {
            let mut score = 0;
            let mut matched_patterns = Vec::new();
            for pat in &patterns {
                let re = match Regex::new(pat) {
                    Ok(r) => r,
                    Err(_) => continue,
                };
                for line in &lines {
                    if re.is_match(line) {
                        score += priority;
                        matched_patterns.push(pat.to_string());
                        break;
                    }
                }
            }
            if score > 0 {
                let total_pat = patterns.len().max(3);
                let mut base_confidence = (matched_patterns.len() as f64 / (total_pat as f64 * 0.5)).min(1.0);

                // High-signal boost
                if let Some(hs_patterns) = high_signal.get(lang) {
                    for mp in &matched_patterns {
                        for hs in hs_patterns {
                            if mp == *hs {
                                base_confidence = base_confidence.max(0.65);
                                break;
                            }
                        }
                    }
                }

                scores.insert(lang.to_string(), LangScore {
                    score,
                    matches: matched_patterns.len(),
                    matched_patterns,
                    total_patterns: patterns.len(),
                    confidence: base_confidence,
                });
            }
        }

        // Known binary boost
        let first_line = snippet.lines().next().unwrap_or("").trim();
        let clean_first = Regex::new(r"^(#!.*\n|\s*sudo\s+)").unwrap().replace(first_line, "");
        let lead_token = clean_first.split_whitespace().next().unwrap_or("");
        let lead_token = lead_token.rsplit('/').next().unwrap_or("").to_lowercase();

        if let Some(&(binary_lang, _)) = KNOWN_BINARIES.get(lead_token.as_str()) {
            if let Some(entry) = scores.get_mut(binary_lang) {
                entry.confidence = entry.confidence.max(0.70);
            } else {
                scores.insert(binary_lang.to_string(), LangScore {
                    score: 10,
                    matches: 1,
                    matched_patterns: vec![format!("binary:{}", lead_token)],
                    total_patterns: 1,
                    confidence: 0.75,
                });
            }
        }

        if scores.is_empty() {
            return DetectionResult {
                language: "Unknown".to_string(),
                confidence: 0.0,
                confidence_label: "UNVERIFIED".to_string(),
                reasoning: "No structural or semantic syntax signatures were detected.".to_string(),
                context: "Agnostic".to_string(),
                color: "#666666".to_string(),
                alternatives: vec![],
            };
        }

        let mut ranked: Vec<(String, LangScore)> = scores.into_iter().collect();
        ranked.sort_by(|a, b| b.1.score.cmp(&a.1.score));

        let primary = &ranked[0];
        let confidence = (primary.1.confidence * 100.0).round() / 100.0;
        let conf_label = if confidence >= 0.8 {
            "HIGH_AUTHORITY"
        } else if confidence >= 0.5 {
            "VERIFIED_DOMAIN"
        } else {
            "LOW_CONFIDENCE"
        };

        let reasoning = if primary.1.matches > 0 {
            format!("Matched {} deterministic syntax fingerprints (e.g., {}).", primary.1.matches, primary.1.matched_patterns[0])
        } else {
            "Inferred via secondary heuristics.".to_string()
        };

        let context_map: HashMap<&str, &str> = [
            ("Python", "Standard Library / CPython"),
            ("Bash", "Unix Shell Environment"),
            ("PowerShell", "Windows Management Framework"),
            ("SQL", "Relational Database Engine"),
            ("JavaScript", "Browser Engine / Node.js Runtime"),
            ("Docker", "Container Daemon"),
            ("Kubernetes", "Cluster Control Plane"),
            ("Git", "Version Control System"),
            ("Rust", "Rustc Compilation Environment"),
        ].into_iter().collect();

        let color = DOMAIN_COLORS.get(primary.0.as_str()).unwrap_or(&"#C9A84C").to_string();

        let alternatives = ranked.iter().skip(1).take(3).map(|(lang, data)| AltDetection {
            language: lang.clone(),
            confidence: (data.confidence * 100.0).round() / 100.0,
        }).collect();

        DetectionResult {
            language: primary.0.clone(),
            confidence,
            confidence_label: conf_label.to_string(),
            reasoning,
            context: context_map.get(primary.0.as_str()).unwrap_or(&"Runtime Environment").to_string(),
            color,
            alternatives,
        }
    }

    // ─── Safety Check ──────────────────────────────────────

    pub fn check_safety(&self, snippet: &str) -> SafetyResult {
        let mut warnings = Vec::new();
        for (pat, desc) in DANGEROUS_PATTERNS.iter() {
            if let Ok(re) = Regex::new(pat) {
                if re.is_match(snippet) {
                    warnings.push(SafetyWarning { level: "danger".to_string(), message: desc.to_string() });
                }
            }
        }

        let mut safe_notes = Vec::new();
        for (pat, desc) in SAFE_PATTERNS.iter() {
            if let Ok(re) = Regex::new(pat) {
                if re.is_match(snippet) {
                    safe_notes.push(desc.to_string());
                }
            }
        }

        if !warnings.is_empty() {
            SafetyResult { level: "DESTRUCTIVE".to_string(), warnings, safe_notes: vec![] }
        } else if !safe_notes.is_empty() {
            SafetyResult { level: "SAFE".to_string(), warnings: vec![], safe_notes }
        } else {
            let first_line = snippet.lines().next().unwrap_or("").trim();
            let clean = Regex::new(r"^\s*sudo\s+").unwrap().replace(first_line, "");
            let lead = clean.split_whitespace().next().unwrap_or("");
            let lead = lead.rsplit('/').next().unwrap_or("").to_lowercase();
            let grounded = SAFETY_CONTEXT.get(lead.as_str()).unwrap_or(
                &"This operation does not match any known safe or dangerous pattern. Review the command and verify the target before executing."
            ).to_string();
            SafetyResult { level: "CAUTION".to_string(), warnings: vec![], safe_notes: vec![grounded] }
        }
    }

    // ─── FTS5 Search ───────────────────────────────────────

    pub fn search(&self, query: &str, limit: usize) -> SearchResult {
        let clean_query = self.sanitize_fts_query(query);
        if clean_query.is_empty() {
            return SearchResult { query: query.to_string(), results: vec![], count: 0 };
        }

        let conn = match self.conn() {
            Ok(c) => c,
            Err(_) => return SearchResult { query: query.to_string(), results: vec![], count: 0 },
        };

        let mut results = Vec::new();
        let mut seen_ids = HashSet::new();

        // Global FTS5 search
        let sql = "SELECT f.id, f.content, f.source, f.tier, rank FROM bible_fts fts JOIN fragments f ON f.rowid = fts.rowid WHERE bible_fts MATCH ? ORDER BY rank LIMIT ?";
        if let Ok(mut stmt) = conn.prepare(sql) {
            let rows = stmt.query_map([&clean_query, &(limit as i64).to_string()], |row| {
                Ok(FragmentResult {
                    id: row.get(0)?,
                    content: row.get::<_, String>(1)?.chars().take(500).collect(),
                    source: row.get(2)?,
                    tier: row.get::<_, Option<String>>(3)?.unwrap_or_else(|| "Derived".to_string()),
                    relevance: row.get::<_, f64>(4)?.abs(),
                })
            });
            if let Ok(rows) = rows {
                for row in rows.flatten() {
                    seen_ids.insert(row.id.clone());
                    results.push(row);
                    if results.len() >= limit { break; }
                }
            }
        }

        SearchResult { query: query.to_string(), count: results.len(), results }
    }

    pub fn search_with_language(&self, query: &str, limit: usize, language: Option<&str>) -> Vec<FragmentResult> {
        let clean_query = self.sanitize_fts_query(query);
        if clean_query.is_empty() {
            return vec![];
        }

        let conn = match self.conn() {
            Ok(c) => c,
            Err(_) => return vec![],
        };

        let mut results = Vec::new();
        let mut seen_ids = HashSet::new();

        if let Some(lang) = language {
            if let Some(patterns) = LANGUAGE_SOURCE_MAP.get(lang) {
                let source_clauses: Vec<String> = patterns.iter().map(|_| "f.source LIKE ?".to_string()).collect();
                let sql = format!(
                    "SELECT f.id, f.content, f.source, f.tier, rank FROM bible_fts fts JOIN fragments f ON f.rowid = fts.rowid WHERE bible_fts MATCH ? AND ({}) ORDER BY rank LIMIT ?",
                    source_clauses.join(" OR ")
                );
                let mut params: Vec<String> = vec![clean_query.clone()];
                for p in patterns.iter() { params.push(p.to_string()); }
                params.push(limit.to_string());
                if let Ok(mut stmt) = conn.prepare(&sql) {
                    let rows = stmt.query_map(rusqlite::params_from_iter(params.iter()), |row| {
                        Ok(FragmentResult {
                            id: row.get(0)?,
                            content: row.get::<_, String>(1)?.chars().take(500).collect(),
                            source: row.get(2)?,
                            tier: row.get::<_, Option<String>>(3)?.unwrap_or_else(|| self.map_source_to_tier(&row.get::<_, String>(2).unwrap_or_default())),
                            relevance: row.get::<_, f64>(4)?.abs(),
                        })
                    });
                    if let Ok(rows) = rows {
                        for row in rows.flatten() {
                            seen_ids.insert(row.id.clone());
                            results.push(row);
                        }
                    }
                }
            }
        }

        // Fill remaining slots with global search
        let remaining = limit.saturating_sub(results.len());
        if remaining > 0 {
            let sql = "SELECT f.id, f.content, f.source, f.tier, rank FROM bible_fts fts JOIN fragments f ON f.rowid = fts.rowid WHERE bible_fts MATCH ? ORDER BY rank LIMIT ?";
            if let Ok(mut stmt) = conn.prepare(sql) {
                let rows = stmt.query_map([&clean_query, &(remaining as i64).to_string()], |row| {
                    Ok(FragmentResult {
                        id: row.get(0)?,
                        content: row.get::<_, String>(1)?.chars().take(500).collect(),
                        source: row.get(2)?,
                        tier: row.get::<_, Option<String>>(3)?.unwrap_or_else(|| "Derived".to_string()),
                        relevance: row.get::<_, f64>(4)?.abs(),
                    })
                });
                if let Ok(rows) = rows {
                    for row in rows.flatten() {
                        if !seen_ids.contains(&row.id) {
                            seen_ids.insert(row.id.clone());
                            results.push(row);
                            if results.len() >= limit { break; }
                        }
                    }
                }
            }
        }

        results.into_iter().take(limit).collect()
    }

    fn sanitize_fts_query(&self, query: &str) -> String {
        let re = Regex::new(r"[a-zA-Z_][\w]*").unwrap();
        let tokens: Vec<String> = re.find_iter(query).map(|m| m.as_str().to_string()).collect();
        if tokens.is_empty() {
            return "".to_string();
        }
        let mut meaningful: Vec<String> = tokens.iter()
            .filter(|t| !SYNTAX_NOISE.contains(t.to_lowercase().as_str()) && t.len() > 1)
            .cloned()
            .collect();
        if meaningful.is_empty() {
            meaningful = tokens.into_iter().filter(|t| t.len() > 2).take(3).collect();
        }
        if meaningful.is_empty() {
            return "".to_string();
        }
        meaningful.truncate(10);
        meaningful.join(" OR ")
    }

    fn map_source_to_tier(&self, source: &str) -> String {
        let s = source.to_lowercase();
        if ["docs.python.org", "developer.mozilla.org", "man7.org", "gnu.org", "postgresql.org/docs"].iter().any(|x| s.contains(x)) {
            "Official".to_string()
        } else if ["man page", "man", "tldr"].iter().any(|x| s.contains(x)) {
            "Man".to_string()
        } else {
            "Derived".to_string()
        }
    }

    // ─── Keyword Extraction ────────────────────────────────

    pub fn extract_keywords(&self, snippet: &str) -> Vec<String> {
        let mut cleaned = Regex::new(r"#.*$").unwrap().replace_all(snippet, "").to_string();
        cleaned = Regex::new(r"//.*$").unwrap().replace_all(&cleaned, "").to_string();
        cleaned = Regex::new(r"/\*.*?\*/").unwrap().replace_all(&cleaned, "").to_string();
        cleaned = Regex::new(r#"[""'].*?[""']"#).unwrap().replace_all(&cleaned, "").to_string();

        let re = Regex::new(r"[a-zA-Z_][\w.]*").unwrap();
        let tokens: Vec<String> = re.find_iter(&cleaned).map(|m| m.as_str().to_string()).collect();

        let mut meaningful = Vec::new();
        let mut seen = HashSet::new();
        for token in tokens {
            let lower = token.to_lowercase();
            if !SYNTAX_NOISE.contains(lower.as_str()) && !seen.contains(&lower) && token.len() > 1 {
                meaningful.push(token.clone());
                seen.insert(lower);
            }
        }
        meaningful.truncate(15);
        meaningful
    }

    pub fn extract_concepts(&self, snippet: &str) -> Vec<String> {
        let mut concepts = Vec::new();
        for (pat, concept) in CONCEPT_PATTERNS.iter() {
            if let Ok(re) = Regex::new(pat) {
                if re.is_match(snippet) {
                    concepts.push(concept.to_string());
                }
            }
        }
        concepts
    }

    pub fn decompose_snippet(&self, snippet: &str) -> Vec<BreakdownItem> {
        let mut breakdown = Vec::new();
        for line in snippet.lines() {
            let line_str = line.trim();
            if line_str.is_empty() { continue; }

            let mut matched_concept = None;
            for (pat, concept) in CONCEPT_PATTERNS.iter() {
                if let Ok(re) = Regex::new(pat) {
                    if re.is_match(line_str) {
                        matched_concept = Some(concept.to_string());
                        break;
                    }
                }
            }

            if let Some(concept) = matched_concept {
                let title = concept.split(' ').map(|w| {
                    let mut c = w.chars();
                    match c.next() {
                        None => String::new(),
                        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                    }
                }).collect::<Vec<_>>().join(" ");
                breakdown.push(BreakdownItem { code: line_str.to_string(), concept: title });
            } else if line_str.len() > 3 && !line_str.starts_with("#") && !line_str.starts_with("//") && !line_str.starts_with("/*") {
                breakdown.push(BreakdownItem { code: line_str.to_string(), concept: "Operation / Declaration".to_string() });
            }
        }
        breakdown
    }

    // ─── Quick Understanding ───────────────────────────────

    pub fn generate_quick_understanding(&self, snippet: &str, lang: &str) -> String {
        let clean = Regex::new(r"^\s*(#!.*\n|\s*sudo\s+)").unwrap().replace(snippet.trim(), "");
        let tokens: Vec<&str> = clean.split_whitespace().collect();
        let lead = tokens.first().map(|s| s.rsplit('/').next().unwrap_or("").to_lowercase()).unwrap_or_default();

        if let Some(&(_, base_summary)) = KNOWN_BINARIES.get(lead.as_str()) {
            return self.enhance_binary_summary(&lead, &tokens, base_summary);
        }

        let lines: Vec<&str> = snippet.trim().lines().collect();
        if lines.len() == 1 {
            return format!("A single {} statement.", if lang.is_empty() || lang == "Unknown" { "code" } else { lang });
        }

        let mut concepts_found = HashSet::new();
        for line in &lines {
            for (pat, concept) in CONCEPT_PATTERNS.iter() {
                if let Ok(re) = Regex::new(pat) {
                    if re.is_match(line.trim()) {
                        concepts_found.insert(*concept);
                        break;
                    }
                }
            }
        }

        if !concepts_found.is_empty() {
            let mut concept_list: Vec<String> = concepts_found.into_iter().map(|s| s.to_string()).collect();
            concept_list.sort();
            let concept_str = concept_list.into_iter().take(3).collect::<Vec<_>>().join(", ");
            return format!("A {} snippet covering {} across {} lines.", lang, concept_str, lines.len());
        }

        format!("A {}-line {} snippet.", lines.len(), if lang.is_empty() || lang == "Unknown" { "code" } else { lang })
    }

    fn enhance_binary_summary(&self, binary: &str, tokens: &[&str], base: &str) -> String {
        let args = if tokens.len() > 1 { &tokens[1..] } else { &[] };
        if args.is_empty() {
            return base.to_string();
        }

        match binary {
            "chmod" => {
                let flags: Vec<&&str> = args.iter().filter(|a| a.starts_with('+') || a.starts_with('-') || a.parse::<u32>().is_ok()).collect();
                let files: Vec<&&str> = args.iter().filter(|a| !a.starts_with('-') && !a.starts_with('+') && a.parse::<u32>().is_err()).collect();
                if flags.iter().any(|f| f.contains("+x")) && !files.is_empty() {
                    return format!("Makes {} executable so it can be run as a script.", files.last().unwrap());
                } else if flags.iter().any(|f| f.contains("777")) && !files.is_empty() {
                    return format!("Sets full read/write/execute permissions for everyone on {}.", files.last().unwrap());
                } else if !files.is_empty() {
                    return format!("Changes file permissions on {}.", files.last().unwrap());
                }
            }
            "git" => {
                let sub = args.first().unwrap_or(&"");
                let sub_map: HashMap<&str, &str> = [
                    ("clone", "Downloads a copy of a remote repository to your machine."),
                    ("commit", "Saves your staged changes as a new commit in the repository."),
                    ("push", "Uploads your local commits to the remote repository."),
                    ("pull", "Downloads and merges remote changes into your current branch."),
                    ("add", "Stages files for the next commit."),
                    ("status", "Shows which files have been modified, staged, or are untracked."),
                    ("log", "Displays the commit history of the repository."),
                    ("diff", "Shows the differences between your working files and the last commit."),
                    ("merge", "Combines another branch's changes into your current branch."),
                    ("rebase", "Replays your commits on top of another branch's history."),
                    ("checkout", "Switches to a different branch or restores files."),
                    ("branch", "Lists, creates, or deletes branches."),
                    ("stash", "Temporarily saves uncommitted changes so you can work on something else."),
                    ("reset", "Undoes commits or unstages files, depending on the flags used."),
                    ("init", "Creates a new Git repository in the current directory."),
                    ("fetch", "Downloads remote changes without merging them."),
                    ("tag", "Creates a named marker for a specific commit (like a version label)."),
                    ("remote", "Manages the list of remote repositories linked to this project."),
                ].into_iter().collect();
                if let Some(&msg) = sub_map.get(sub) { return msg.to_string(); }
                return format!("Runs a Git '{}' operation.", sub);
            }
            "docker" => {
                let sub = args.first().unwrap_or(&"");
                let sub_map: HashMap<&str, &str> = [
                    ("build", "Builds a container image from a Dockerfile."),
                    ("run", "Creates and starts a new container from an image."),
                    ("exec", "Runs a command inside a running container."),
                    ("compose", "Manages multi-container applications defined in docker-compose.yml."),
                    ("pull", "Downloads a container image from a registry."),
                    ("push", "Uploads a container image to a registry."),
                    ("ps", "Lists currently running containers."),
                    ("stop", "Stops a running container."),
                    ("rm", "Removes a stopped container."),
                    ("logs", "Shows the output logs of a container."),
                    ("images", "Lists all downloaded container images."),
                ].into_iter().collect();
                if let Some(&msg) = sub_map.get(sub) { return msg.to_string(); }
                return format!("Runs a Docker '{}' command.", sub);
            }
            "kubectl" => {
                let sub = args.first().unwrap_or(&"");
                let sub_map: HashMap<&str, &str> = [
                    ("get", "Retrieves information about Kubernetes resources."),
                    ("apply", "Creates or updates resources from a configuration file."),
                    ("delete", "Removes resources from the cluster."),
                    ("describe", "Shows detailed information about a specific resource."),
                    ("logs", "Displays logs from a container in a pod."),
                    ("exec", "Runs a command inside a container in a pod."),
                    ("scale", "Changes the number of replicas for a deployment."),
                    ("port-forward", "Forwards a local port to a port on a pod."),
                ].into_iter().collect();
                if let Some(&msg) = sub_map.get(sub) { return msg.to_string(); }
                return format!("Runs a kubectl '{}' operation on the cluster.", sub);
            }
            "pip" | "pip3" => {
                let sub = args.first().unwrap_or(&"");
                if sub == &"install" {
                    let pkgs: Vec<&&str> = args.iter().skip(1).filter(|a| !a.starts_with('-')).collect();
                    if !pkgs.is_empty() {
                        return format!("Installs the Python package{} {}.", if pkgs.len() > 1 { "s" } else { "" }, pkgs.iter().take(3).map(|&&s| s).collect::<Vec<&str>>().join(", "));
                    }
                    return "Installs Python packages.".to_string();
                } else if sub == &"uninstall" {
                    return "Removes a Python package from the current environment.".to_string();
                } else if sub == &"freeze" {
                    return "Lists all installed Python packages and their versions.".to_string();
                }
            }
            "npm" | "yarn" => {
                let sub = args.first().unwrap_or(&"");
                if sub == &"install" || sub == &"add" {
                    let pkgs: Vec<&&str> = args.iter().skip(1).filter(|a| !a.starts_with('-')).collect();
                    if !pkgs.is_empty() {
                        return format!("Installs the package{} {} into the project.", if pkgs.len() > 1 { "s" } else { "" }, pkgs.iter().take(3).map(|&&s| s).collect::<Vec<&str>>().join(", "));
                    }
                    return "Installs project dependencies from package.json.".to_string();
                } else if sub == &"run" {
                    let script = args.get(1).unwrap_or(&"a");
                    return format!("Runs the '{}' script defined in package.json.", script);
                } else if sub == &"init" {
                    return "Creates a new package.json file for the project.".to_string();
                }
            }
            _ => {}
        }

        base.to_string()
    }

    // ─── Full Analysis ─────────────────────────────────────

    pub fn analyze(&self, snippet: &str) -> AnalysisResult {
        let detection = self.detect_language(snippet);
        let detected_lang = detection.language.clone();
        let lang_param = if detected_lang == "Unknown" { None } else { Some(detected_lang.as_str()) };

        let keywords = self.extract_keywords(snippet);
        let concepts = self.extract_concepts(snippet);

        let mut search_terms = keywords.clone();
        search_terms.extend(concepts.clone());
        search_terms.truncate(9);

        let search_query = if search_terms.is_empty() {
            "".to_string()
        } else {
            search_terms.join(" OR ")
        };

        let mut results = if search_query.is_empty() {
            vec![]
        } else {
            self.search_with_language(&search_query, 15, lang_param)
        };

        // Fallback: syntax terms inside domain
        if results.len() < 5 && lang_param.is_some() {
            let re = Regex::new(r"[a-zA-Z_][\w]*").unwrap();
            let raw_tokens: Vec<String> = re.find_iter(snippet).map(|m| m.as_str().to_string()).collect();
            let syntax_in_domain: Vec<String> = raw_tokens.iter()
                .filter(|t| SYNTAX_NOISE.contains(t.to_lowercase().as_str()) && t.len() > 2)
                .cloned()
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .take(5)
                .collect();
            if !syntax_in_domain.is_empty() {
                let domain_q = syntax_in_domain.into_iter().chain(keywords.iter().cloned().take(3)).collect::<Vec<_>>().join(" OR ");
                let domain_results = self.search_with_language(&domain_q, 10, lang_param);
                let seen: HashSet<String> = results.iter().map(|r| r.id.clone()).collect();
                for r in domain_results {
                    if !seen.contains(&r.id) {
                        results.push(r);
                    }
                }
            }
        }

        // Final fallback: raw identifiers > 3 chars
        if results.is_empty() && lang_param.is_some() {
            let re = Regex::new(r"[a-zA-Z_][\w]*").unwrap();
            let raw_tokens: Vec<String> = re.find_iter(snippet).map(|m| m.as_str().to_string()).collect();
            let raw_meaningful: Vec<String> = raw_tokens.into_iter().filter(|t| t.len() > 3).take(5).collect();
            if !raw_meaningful.is_empty() {
                let fallback_q = raw_meaningful.join(" OR ");
                results = self.search_with_language(&fallback_q, 10, lang_param);
            }
        }

        results.truncate(15);

        let safety = self.check_safety(snippet);
        let first_line = snippet.lines().next().unwrap_or("").trim();
        let first_line = if first_line.len() > 80 { format!("{}...", &first_line[..77]) } else { first_line.to_string() };
        let quick_understanding = self.generate_quick_understanding(snippet, &detected_lang);
        let breakdown = self.decompose_snippet(snippet);
        let total = self.total_fragments();

        let result_count = results.len();
        AnalysisResult {
            input: first_line,
            language: detection,
            safety,
            keywords: keywords.into_iter().chain(concepts.into_iter()).collect(),
            breakdown,
            quick_understanding,
            results,
            result_count,
            total_fragments: total,
        }
    }

    // ─── Stats ─────────────────────────────────────────────

    pub fn get_stats(&self) -> StatsResult {
        let conn = match self.conn() {
            Ok(c) => c,
            Err(_) => return StatsResult { total_fragments: 0, domains: vec![] },
        };

        let total: i64 = conn.query_row("SELECT COUNT(*) FROM fragments", [], |row| row.get(0)).unwrap_or(0);
        let threshold = (total as f64 * 0.005).max(50.0) as i64;

        let sql = r#"
            SELECT
                CASE
                    WHEN source LIKE 'python/%'      OR source LIKE '%docs.python.org%'                      THEN 'Python'
                    WHEN source LIKE 'javascript/%'  OR source LIKE '%nodejs%' OR source LIKE '%npmjs%'      THEN 'JavaScript'
                    WHEN source LIKE 'typescript/%'  OR source LIKE '%typescriptlang%'                       THEN 'TypeScript'
                    WHEN source LIKE 'rust/%'        OR source LIKE '%doc.rust-lang%' OR source LIKE '%docs.rs%' THEN 'Rust'
                    WHEN source LIKE 'golang/%'      OR source LIKE '%go.dev%' OR source LIKE '%pkg.go.dev%' THEN 'Go'
                    WHEN source LIKE 'ruby/%'        OR source LIKE '%ruby%'                                 THEN 'Ruby'
                    WHEN source LIKE 'php/%'         OR source LIKE '%php.net%'                              THEN 'PHP'
                    WHEN source LIKE 'java/%'        AND source NOT LIKE '%javascript%'
                                                     AND source NOT LIKE '%typescript%'                      THEN 'Java'
                    WHEN source LIKE 'bash/%'        OR source LIKE '%gnu.org/software/bash%'
                                                     OR source LIKE '%tldp.org%'                             THEN 'Bash'
                    WHEN source LIKE 'powershell/%'  OR source LIKE '%microsoft.com/powershell%'             THEN 'PowerShell'
                    WHEN source LIKE 'sql/%'         OR source LIKE '%mysql%' OR source LIKE '%mariadb%'
                                                     OR source LIKE '%postgresql%' OR source LIKE '%postgres%'
                                                     OR source LIKE '%sqlite%'                               THEN 'SQL'
                    WHEN source LIKE 'docker/%'      OR source LIKE '%docker.com%'                           THEN 'Docker'
                    WHEN source LIKE 'kubernetes/%'  OR source LIKE '%k8s%'                                  THEN 'Kubernetes'
                    WHEN source LIKE 'nginx/%'       OR source LIKE '%nginx.org%'                            THEN 'Nginx'
                    WHEN source LIKE 'systemd/%'     OR source LIKE '%freedesktop.org%'                      THEN 'systemd'
                    WHEN source LIKE 'git/%'         OR source LIKE '%git-scm%'                              THEN 'Git'
                    WHEN source LIKE 'ansible/%'     OR source LIKE '%ansible.com%'                          THEN 'Ansible'
                    WHEN source LIKE 'css/%'         OR source LIKE '%/CSS/%'                                THEN 'CSS'
                    WHEN source LIKE 'html/%'        OR source LIKE '%/HTML/%'                               THEN 'HTML'
                    WHEN source LIKE 'yaml/%'                                                                THEN 'YAML'
                    WHEN source LIKE 'json/%'                                                                THEN 'JSON'
                    WHEN source LIKE 'terraform/%'   OR source LIKE '%hashicorp%'                            THEN 'Terraform'
                    WHEN source LIKE 'csharp/%'      OR source LIKE '%dotnet/csharp%'                        THEN 'C#'
                    WHEN source LIKE 'kotlin/%'      OR source LIKE '%kotlinlang%'                           THEN 'Kotlin'
                    WHEN source LIKE 'c/%'           OR source LIKE 'cpp/%' OR source LIKE '%cppreference%'  THEN 'C/C++'
                    WHEN source LIKE 'swift/%'       OR source LIKE '%swift.org%'                            THEN 'Swift'
                    WHEN source LIKE 'linux/%'       OR source LIKE '%linux%' OR source LIKE '%man7.org%'
                                                     OR source LIKE '%sourceware.org%'                       THEN 'Linux'
                    WHEN source LIKE 'node/%'        OR source LIKE '%node.js%'                              THEN 'Node.js'
                    WHEN source LIKE 'mysql/%'                                                               THEN 'MySQL'
                    WHEN source LIKE 'postgresql/%'  OR source LIKE '%postgres%'                             THEN 'PostgreSQL'
                    WHEN source LIKE 'linux-tools/%' OR source LIKE '%binutils%'                             THEN 'Linux'
                    ELSE 'Other'
                END as domain,
                COUNT(*) as count
            FROM fragments
            GROUP BY domain
            ORDER BY count DESC
        "#;

        let mut domains = Vec::new();
        if let Ok(mut stmt) = conn.prepare(sql) {
            let rows = stmt.query_map([], |row| {
                let name: String = row.get(0)?;
                let count: i64 = row.get(1)?;
                let color = DOMAIN_COLORS.get(name.as_str()).unwrap_or(&"#666").to_string();
                Ok(DomainStat { name, count, color })
            });
            if let Ok(rows) = rows {
                for row in rows.flatten() {
                    if row.count >= threshold || row.name != "Other" {
                        domains.push(row);
                    }
                }
            }
        }

        StatsResult { total_fragments: total, domains }
    }
}

// ─── Internal Types ────────────────────────────────────────

struct LangScore {
    score: i32,
    matches: usize,
    matched_patterns: Vec<String>,
    total_patterns: usize,
    confidence: f64,
}
