pub mod nom;
pub mod utils;

use std::fmt::Display;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct YarInclude {
    pub value: String
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct YarImport {
    pub value: String
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
    StringCount(String),
    RuleRef(String),
    BytesWithOffset(String, Box<YarRuleConditionNode>),
    Reserved(String),
    ConstString(String),
    Regex(String),
    Number(i64),
    Size(usize),
    None(String),
}

impl Display for YarRuleConditionNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self{
            Self::And(a, b) => write!(f, "({} and {})", a, b),
            Self::Or(a, b) => write!(f, "({} or {})", a, b),
            Self::At(a, b) => write!(f, "({} at {})", a, b),
            Self::Of(a, b) => write!(f, "({} of {})", a, b),
            Self::In(a, b) => write!(f, "({} in {})", a, b),
            Self::Range(a, b) => write!(f, "({} .. {})", a, b),
            Self::Set(a) => write!(f, "({})", a.iter().map(|s| s.to_string()).collect::<Vec<String>>().join(", ")),
            Self::Cmp(op, a, b) => write!(f, "({} {} {})", a, op, b),
            Self::Arithm(op, a, b) => write!(f, "({} {} {})", a, op, b),
            Self::Not(a) => write!(f, "(not {})", a),
            Self::StringRef(a) => write!(f, "({})", a),
            Self::StringCount(a) => write!(f, "({})", a),
            Self::ImportRef(a) => write!(f, "({})", a),
            Self::RuleRef(a) => write!(f, "({})", a),
            Self::Reserved(a) => write!(f, "({})", a),
            Self::ConstString(a) => write!(f, "\"{}\"", a),
            Self::Regex(a) => write!(f, "/{}/", a),
            Self::Number(a) => write!(f, "({})", a),
            Self::Size(a) => write!(f, "({})", a),
            Self::BytesWithOffset(a, b) => write!(f, "{}({})", a, b),
            Self::None(s) => write!(f, "({})", s),
        }
    }
}

#[derive(Debug)]
pub struct YarRuleBody {
    pub meta: std::collections::HashMap<String, String>,
    pub strings: Vec<(String, String)>,
    pub condition: YarRuleConditionNode,
}

impl Display for YarRuleBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.meta.len() > 0{
            write!(f, "meta:\n")?;
            for (n, m) in &self.meta{
                write!(f, "\t{} = {}\n", n, m)?;
            }
        }
        if self.strings.len() > 0{
            write!(f, "strings:\n")?;
            for (n, m) in &self.strings{
                write!(f, "\t{} = {}\n", n, m)?;
            }
        }
        write!(f, "condition:\n")?;
        write!(f, "\t{}\n", self.condition)
    }
}

#[derive(Debug)]
pub struct YarRule {
    pub private: bool,
    pub name: String,
    pub tags: Vec<String>,
    pub body: YarRuleBody,
    pub refs: std::collections::HashSet<String>
}

impl Display for YarRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "rule {}{} {{\n{}\n}}", self.name, if self.tags.len()>0 {format!(":{}", self.tags.join(" "))} else {"".to_string()}, self.body)
    }
}

impl YarRule{
    pub fn order(&self, set: &YarRuleSet) -> Vec<String>{
        let mut res = vec![];
        for ii in &self.refs{
            for iii in &set.rules.get(ii).unwrap().order(set){
                if res.contains(iii){
                    res.push(iii.to_string());
                }
            }
            if res.contains(ii){
                res.push(ii.to_string());
            }
        }
        res
    }
}

#[derive(Default, Debug)]
pub struct YarRuleSet {
    pub includes: Vec<YarInclude>,
    pub imports: Vec<YarImport>,
    pub rules: std::collections::HashMap<String, YarRule>
}

impl YarRuleSet {
    pub fn new(includes: Vec<YarInclude>, imports: Vec<YarImport>, mut rules: std::collections::HashMap<String, YarRule>) -> YarRuleSet{
        for r in rules{
            r.set_refs();
        }
        YarRuleSet{
            includes,
            imports,
            rules
        }
    }

    pub fn order(&self) -> Vec<String>{
        let mut res = vec![];
        for (_, r) in &self.rules{
            if r.refs.len() > 0{
                for ii in r.order(self){
                    if res.contains(&ii){
                        res.push(ii);
                    }
                }
            }
            if res.contains(&r.name){
                res.push(r.name.to_string());
            }
        }
        res
    }
}

impl Display for YarRuleSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        println!("{:#?}", self);
        let vv = self.order();
        println!("{:?}", vv);
        for v in vv{
            write!(f, "{}\n", self.rules.get(&v).unwrap())?;
        }
        write!(f, "\n")
    }
}
