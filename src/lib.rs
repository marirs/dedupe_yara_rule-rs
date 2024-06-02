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
    /// Formats the value of the struct and writes it to the given formatter.
    ///
    /// # Arguments
    ///
    /// * `f` - A mutable reference to the formatter to write the formatted value to.
    ///
    /// # Return
    ///
    /// This method returns a `std::fmt::Result`, indicating whether the operation was successful.
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
    /// Formats the given expression as a string.
    ///
    /// This method is used to format expressions in a user-friendly way. It takes a formatter
    /// and writes the formatted expression to it.
    ///
    /// Arguments
    /// - `f`: A mutable reference to a `std::fmt::Formatter`.
    ///        The formatter to write the formatted expression to.
    ///
    /// Returns
    /// - `std::fmt::Result`: A `Result` indicating whether the formatting was successful or not.
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

/// `YarRuleBody` is a struct that represents the body of a Yara rule.
/// It contains metadata, strings, and a condition for the rule.
///
/// # Fields
/// - `meta`: A `HashMap` of metadata key-value pairs associated with the rule.
/// - `strings`: A vector of tuples, each containing a string identifier and its value.
/// - `condition`: An instance of `YarRuleConditionNode` representing the condition for the rule.
#[derive(Debug, Clone)]
pub struct YarRuleBody {
    /// Rule Meta
    pub meta: HashMap<String, String>,
    /// Rule identifiers and values
    pub strings: Vec<(String, String)>,
    /// Rule condition
    pub condition: YarRuleConditionNode,
}

impl Display for YarRuleBody {
    /// Formats the struct into a string representation.
    ///
    /// This method formats the struct into a string representation and writes it to the provided `std::fmt::Formatter`.
    ///
    /// # Arguments
    ///
    /// * `f` - A mutable reference to a `std::fmt::Formatter` object.
    ///
    /// # Errors
    ///
    /// This method returns a `std::fmt::Result` indicating whether the formatting succeeded or not.
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

/// `YarRule` is a struct representing a Yara rule.
///
/// A Yara rule consists of a private flag, a global flag, a name, tags, a body,
/// a set of references, and an added flag.
///
/// # Fields
///
/// - `private`: Indicates whether the rule is private.
/// - `global`: Indicates whether the rule is global.
/// - `name`: The name of the rule.
/// - `tags`: A vector of tags associated with the rule.
/// - `body`: An instance of `YarRuleBody` representing the body of the rule.
/// - `refs`: A `HashSet` containing references used in the rule.
/// - `added`: Indicates whether the rule has been added.
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
    /// Formats the rule object into a string representation.
    ///
    /// # Arguments
    ///
    /// * `f` - A mutable reference to a Formatter to write the formatted string to.
    ///
    /// # Errors
    ///
    /// Returns a Result indicating whether the operation was successful or not.
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
    /// Creates a new `YarRule` instance.
    ///
    /// # Arguments
    ///
    /// * `private` - A boolean indicating whether the rule is private.
    /// * `global` - A boolean indicating whether the rule is global.
    /// * `name` - The name of the rule.
    /// * `tags` - A vector of tags associated with the rule.
    /// * `body` - The body of the rule.
    ///
    /// # Returns
    ///
    /// A new `YarRule` instance.
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

/// `YarRuleSet` is a struct representing a set of Yara rules.
/// It contains the name of the rule set, includes, imports, rules, and references.
///
/// # Fields
/// - `name`: A string representing the name of the rule set.
/// - `includes`: A vector of `YarInclude` struct representing the Yara includes for the rule set.
/// - `imports`: A vector of `YarImport` struct representing the Yara imports for the rule set.
/// - `rules`: A `HashMap` where the key is the rule name and the value is a `YarRule` struct representing the rule.
/// - `refs`: A vector of strings representing the references used in the rule set.
#[derive(Default, Debug)]
pub struct YarRuleSet {
    pub name: String,
    pub includes: Vec<YarInclude>,
    pub imports: Vec<YarImport>,
    pub rules: HashMap<String, YarRule>,
    pub refs: Vec<String>,
}

impl YarRuleSet {
    /// Creates a new `YarRuleSet` with the given parameters.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the rule set.
    /// * `includes` - The list of `YarInclude` objects to include in the rule set.
    /// * `imports` - The list of `YarImport` objects to import in the rule set.
    /// * `rules` - The map of rule names to `YarRule` objects.
    ///
    /// # Returns
    ///
    /// The newly created `YarRuleSet`.
    ///
    /// # Example
    ///
    /// ```norun
    /// use crate::YarRuleSet;
    /// use crate::YarInclude;
    /// use crate::YarImport;
    /// use crate::YarRule;
    /// use std::collections::HashMap;
    ///
    /// let includes = vec![YarInclude {}];
    /// let imports = vec![YarImport {}];
    /// let rules = HashMap::new();
    ///
    /// let rule_set = YarRuleSet::new("My Rule Set".to_string(), includes, imports, rules);
    /// ```
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
    /// Formats the rules of an object and writes them to a formatter.
    ///
    /// # Arguments
    ///
    /// - `self`: A reference to the current object.
    /// - `f`: A mutable reference to a `std::fmt::Formatter` where the formatted rules will be written.
    ///
    /// # Returns
    ///
    /// This method returns a `std::fmt::Result` indicating whether the writing operation was successful or not.
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

/// `YarAll` is a struct that represents a collection of Yara rules and their associated imports.
/// It consolidates all Yara rules and their imports collected from the input paths.
#[derive(Default, Debug)]
pub struct YarAll {
    /// All imports collected from all the rules.
    /// This will sit at the top of the final output.
    pub imports: HashSet<String>,
    /// All rules collected from all the rules.
    pub rules: Vec<YarRule>,
}

impl YarAll {
    /// Creates a new YarAll instance.
    ///
    /// # Arguments
    ///
    /// * `sets` - A HashMap containing YarRuleSets, where the key is the name of the ruleset.
    /// * `skip_rules` - A Vec of strings containing the names of rules that should be skipped.
    ///
    /// # Returns
    ///
    /// A YarAll instance containing the imported rules and the ruleset.
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
        YarAll {
            imports,
            rules: yararules,
        }
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
