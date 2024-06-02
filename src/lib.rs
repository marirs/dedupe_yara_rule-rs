pub mod cli;
pub mod nom;
pub mod utils;

use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct YarInclude {
    pub value: String,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct YarImport {
    pub value: String,
}

impl Display for YarImport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[derive(Debug, Clone)]
pub enum YarRuleConditionNode {
    And(Box<YarRuleConditionNode>, Box<YarRuleConditionNode>),
    Or(Box<YarRuleConditionNode>, Box<YarRuleConditionNode>),
    At(Box<YarRuleConditionNode>, Box<YarRuleConditionNode>),
    Of(Box<YarRuleConditionNode>, Box<YarRuleConditionNode>),
    In(Box<YarRuleConditionNode>, Box<YarRuleConditionNode>),
    Range(Box<YarRuleConditionNode>, Box<YarRuleConditionNode>),
    Cmp(String, Box<YarRuleConditionNode>, Box<YarRuleConditionNode>),
    Arithm(String, Box<YarRuleConditionNode>, Box<YarRuleConditionNode>),
    Set(Vec<Box<YarRuleConditionNode>>),
    Not(Box<YarRuleConditionNode>),
    ImportRef(String),
    StringRef(String),
    StringRefMask(String),
    StringCount(String),
    RuleRef(String),
    BytesWithOffset(String, Box<YarRuleConditionNode>),
    Reserved(String),
    ConstString(String),
    Regex(String),
    Number(i64),
    Size(usize),
    Boolean(bool),
    None(String),
    ForIn(
        Box<YarRuleConditionNode>,
        String,
        Box<YarRuleConditionNode>,
        Box<YarRuleConditionNode>,
    ),
    ForOf(
        Box<YarRuleConditionNode>,
        Box<YarRuleConditionNode>,
        Box<YarRuleConditionNode>,
    ),
}

impl Display for YarRuleConditionNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::And(a, b) => write!(f, "{} and {}", a, b),
            Self::Or(a, b) => write!(f, "{} or {}", a, b),
            Self::At(a, b) => write!(f, "{} at {}", a, b),
            Self::Of(a, b) => write!(f, "{} of {}", a, b),
            Self::In(a, b) => write!(f, "{} in {}", a, b),
            Self::Range(a, b) => write!(f, "({} .. {})", a, b),
            Self::Set(a) => write!(
                f,
                "({})",
                a.iter()
                    .map(|s| if let Self::StringRefMask(ss) = &**s {
                        ss.to_string()
                    } else {
                        s.to_string()
                    })
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Self::Cmp(op, a, b) => write!(f, "{} {} {}", a, op, b),
            Self::Arithm(op, a, b) => write!(f, "{} {} {}", a, op, b),
            Self::Not(a) => write!(f, "not {}", a),
            Self::StringRef(a) => write!(f, "{}", a),
            Self::StringRefMask(a) => write!(f, "({})", a),
            Self::StringCount(a) => write!(f, "({})", a),
            Self::ImportRef(a) => write!(f, "{}", a),
            Self::RuleRef(a) => write!(f, "{}", a),
            Self::Reserved(a) => write!(f, "{}", a),
            Self::ConstString(a) => write!(f, "{}", a),
            Self::Regex(a) => write!(f, "{}", a),
            Self::Number(a) => write!(f, "{}", a),
            Self::Size(a) => write!(f, "{}", a),
            Self::BytesWithOffset(a, b) => write!(f, "{}({})", a, b),
            Self::Boolean(b) => write!(f, "{}", if *b { "true" } else { "false" }),
            Self::None(s) => write!(f, "none ({})", s),
            Self::ForIn(expression, identifier, indexes, boolean_expression) => write!(
                f,
                "for {} {} in {} : ({})",
                expression, identifier, indexes, boolean_expression
            ),
            Self::ForOf(expression, set, boolean_expression) => write!(
                f,
                "for {} of {} : ({})",
                expression, set, boolean_expression
            ),
        }
    }
}

impl YarRuleConditionNode {
    pub fn get_refs(&self) -> HashSet<String> {
        let mut res = HashSet::new();
        match self {
            Self::And(a, b) => {
                res.extend(a.get_refs());
                res.extend(b.get_refs());
            }
            Self::Or(a, b) => {
                res.extend(a.get_refs());
                res.extend(b.get_refs());
            }
            Self::At(a, b) => {
                res.extend(a.get_refs());
                res.extend(b.get_refs());
            }
            Self::Of(a, b) => {
                res.extend(a.get_refs());
                res.extend(b.get_refs());
            }
            Self::In(a, b) => {
                res.extend(a.get_refs());
                res.extend(b.get_refs());
            }
            Self::Range(a, b) => {
                res.extend(a.get_refs());
                res.extend(b.get_refs());
            }
            Self::Set(a) => {
                for aa in a {
                    res.extend(aa.get_refs());
                }
            }
            Self::Cmp(_op, a, b) => {
                res.extend(a.get_refs());
                res.extend(b.get_refs());
            }
            Self::Arithm(_op, a, b) => {
                res.extend(a.get_refs());
                res.extend(b.get_refs());
            }
            Self::Not(a) => {
                res.extend(a.get_refs());
            }
            Self::StringRef(_a) => {}
            Self::StringRefMask(_a) => {}
            Self::StringCount(_a) => {}
            Self::ImportRef(_a) => {}
            Self::RuleRef(a) => {
                res.insert(a.clone());
            }
            Self::Reserved(_a) => {}
            Self::ConstString(_a) => {}
            Self::Regex(_a) => {}
            Self::Number(_a) => {}
            Self::Size(_a) => {}
            Self::BytesWithOffset(_a, b) => {
                res.extend(b.get_refs());
            }
            Self::Boolean(_a) => {}
            Self::None(_s) => {}
            Self::ForIn(expression, _identifier, indexes, boolean_expression) => {
                res.extend(expression.get_refs());
                res.extend(indexes.get_refs());
                res.extend(boolean_expression.get_refs());
            }
            Self::ForOf(expression, set, boolean_expression) => {
                res.extend(expression.get_refs());
                res.extend(set.get_refs());
                res.extend(boolean_expression.get_refs());
            }
        }
        res
    }
}

#[derive(Debug, Clone)]
pub struct YarRuleBody {
    pub meta: HashMap<String, String>,
    pub strings: Vec<(String, String)>,
    pub condition: YarRuleConditionNode,
}

impl Display for YarRuleBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.meta.is_empty() {
            writeln!(f, "meta:")?;
            for (n, m) in &self.meta {
                writeln!(f, "\t{} = {}", n, m)?;
            }
        }
        if !self.strings.is_empty() {
            writeln!(f, "strings:")?;
            for (n, m) in &self.strings {
                writeln!(f, "\t{} = {}", n, m)?;
            }
        }
        writeln!(f, "condition:")?;
        writeln!(f, "\t{}", self.condition)
    }
}

#[derive(Debug, Clone)]
pub struct YarRule {
    pub private: bool,
    pub global: bool,
    pub name: String,
    pub tags: Vec<String>,
    pub body: YarRuleBody,
    pub refs: HashSet<String>,
    pub added: bool,
}

impl Display for YarRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}rule {}{} {{\n{}}}\n",
            if self.global { "global " } else { "" },
            if self.private { "private " } else { "" },
            self.name,
            if !self.tags.is_empty() {
                format!(":{}", self.tags.join(" "))
            } else {
                "".to_string()
            },
            self.body
        )
    }
}

impl YarRule {
    pub fn new(
        private: bool,
        global: bool,
        name: String,
        tags: Vec<String>,
        body: YarRuleBody,
    ) -> YarRule {
        YarRule {
            private,
            global,
            name,
            tags,
            body,
            refs: HashSet::new(),
            added: false,
        }
    }

    pub fn get_rule_refs(&self) -> HashSet<String> {
        self.body.condition.get_refs()
    }
}

#[derive(Default, Debug)]
pub struct YarRuleSet {
    pub name: String,
    pub includes: Vec<YarInclude>,
    pub imports: Vec<YarImport>,
    pub rules: HashMap<String, YarRule>,
    pub refs: Vec<String>,
}

impl YarRuleSet {
    pub fn new(
        name: String,
        includes: Vec<YarInclude>,
        imports: Vec<YarImport>,
        rules: HashMap<String, YarRule>,
    ) -> YarRuleSet {
        YarRuleSet {
            name,
            includes,
            imports,
            rules,
            refs: vec![],
        }
    }
}

impl Display for YarRuleSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for v in self.rules.values() {
            writeln!(f, "{}", v)?;
        }
        writeln!(f)
    }
}

impl Ord for YarRuleSet {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.refs.contains(&other.name) {
            std::cmp::Ordering::Less
        } else if other.refs.contains(&self.name) {
            std::cmp::Ordering::Greater
        } else if self.refs.len() > other.refs.len() {
            std::cmp::Ordering::Less
        } else if self.refs.len() < other.refs.len() {
            std::cmp::Ordering::Greater
        } else {
            std::cmp::Ordering::Equal
        }
    }
}

impl PartialOrd for YarRuleSet {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for YarRuleSet {
    fn eq(&self, other: &Self) -> bool {
        !(self.refs.contains(&other.name) || other.refs.contains(&self.name))
    }
}

impl Eq for YarRuleSet {}

#[derive(Default, Debug)]
pub struct YarAll {
    pub imports: HashSet<String>,
    pub rules: Vec<YarRule>,
}

impl YarAll {
    pub fn new(sets: HashMap<String, YarRuleSet>, skip_rules: Vec<String>) -> YarAll {
        let mut imports = HashSet::<String>::new();
        let mut ruleset = HashMap::<String, YarRule>::new();
        let mut total_yara_rules = 0;
        let mut ignored = 0;
        for s in sets.values() {
            for i in &s.imports {
                imports.insert(i.value.clone());
            }
            for r in s.rules.values() {
                total_yara_rules += 1;
                if !skip_rules.contains(&r.name) {
                    ruleset.insert(r.name.clone(), r.clone());
                } else {
                    ignored += 1;
                }
            }
        }
        println!("* Total yara rules: {}", total_yara_rules);
        println!("! Total rules to skip: {}", skip_rules.len());
        println!(
            "* Total yara rules after dedupe: {} ({}%)",
            ruleset.len(),
            100 * ruleset.len() / total_yara_rules
        );
        println!("! Total skipped rules: {}", ignored);
        let mut refs = HashMap::new();
        for (n, yr) in &ruleset {
            for i in yr.get_rule_refs() {
                if i != *n {
                    let ss = refs.entry(i).or_insert(vec![]);
                    ss.push(n.to_string())
                }
            }
        }

        for (n, v) in &refs {
            if let Some(yr) = ruleset.get_mut(n) {
                yr.refs = v.clone().into_iter().collect()
            }
        }
        let mut yararules = ruleset.into_values().collect::<Vec<YarRule>>();
        yararules.sort_by(|a, b| b.refs.len().cmp(&a.refs.len()));
        YarAll { imports, rules: yararules }
    }
}

impl Display for YarAll {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for r in &self.rules {
            writeln!(f, "{}", r)?;
        }
        writeln!(f)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    #[test]
    fn check_rule_ordering() {
        let mut rules_map = HashMap::new();
        rules_map.insert(
            "rule1".to_string(),
            super::YarRule::new(
                false,
                false,
                "rule1".to_string(),
                vec![],
                super::YarRuleBody {
                    meta: HashMap::new(),
                    strings: vec![],
                    condition: super::YarRuleConditionNode::RuleRef("rule2".to_string()),
                },
            ),
        );
        rules_map.insert(
            "rule2".to_string(),
            super::YarRule::new(
                false,
                false,
                "rule2".to_string(),
                vec![],
                super::YarRuleBody {
                    meta: HashMap::new(),
                    strings: vec![],
                    condition: super::YarRuleConditionNode::RuleRef("rule3".to_string()),
                },
            ),
        );
        rules_map.insert(
            "rule3".to_string(),
            super::YarRule::new(
                false,
                false,
                "rule3".to_string(),
                vec![],
                super::YarRuleBody {
                    meta: HashMap::new(),
                    strings: vec![],
                    condition: super::YarRuleConditionNode::Boolean(false),
                },
            ),
        );

        let yy = super::YarRuleSet::new("rule_set".to_string(), vec![], vec![], rules_map);
        println!("{:#?}", yy);
    }

    #[test]
    fn check_rule_ordering_2() {
        let mut rules_map = HashMap::new();
        rules_map.insert(
            "rule1".to_string(),
            super::YarRule::new(
                false,
                false,
                "rule1".to_string(),
                vec![],
                super::YarRuleBody {
                    meta: HashMap::new(),
                    strings: vec![],
                    condition: super::YarRuleConditionNode::RuleRef("rule3".to_string()),
                },
            ),
        );
        rules_map.insert(
            "rule2".to_string(),
            super::YarRule::new(
                false,
                false,
                "rule2".to_string(),
                vec![],
                super::YarRuleBody {
                    meta: HashMap::new(),
                    strings: vec![],
                    condition: super::YarRuleConditionNode::RuleRef("rule3".to_string()),
                },
            ),
        );
        rules_map.insert(
            "rule3".to_string(),
            super::YarRule::new(
                false,
                false,
                "rule3".to_string(),
                vec![],
                super::YarRuleBody {
                    meta: HashMap::new(),
                    strings: vec![],
                    condition: super::YarRuleConditionNode::Boolean(false),
                },
            ),
        );

        let yy = super::YarRuleSet::new("rule_set".to_string(), vec![], vec![], rules_map);
        println!("{:#?}", yy);
    }
}
