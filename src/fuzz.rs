use std::collections::HashMap;
use super::set::FuzzySet;
use super::common::{
    Category,
    Term,
    FuzzyValue,
    FuzzyResult,
    FuzzyIdent,
    FuzzyError
};

pub struct FuzzerConfig {}
impl FuzzerConfig {
    pub fn new(
    ) -> Self {
        Self {}
    }
}

pub struct Fuzzer {
    categories: HashMap<Category, FuzzySet>,
    outputs: HashMap<Category, FuzzySet>,
    rules: Vec<FuzzyRule>,
    _config: FuzzerConfig
}

impl Fuzzer {
    pub fn new(
    ) -> Self {
        Self {
            categories: HashMap::new(),
            outputs: HashMap::new(),
            rules: Vec::new(),
            _config: FuzzerConfig::new()
        }
    }

    /// Applies rules to input and returns calculated FuzzySets.
    pub fn apply(
        &self,
        values: &HashMap<Category, f64>
    ) -> FuzzyResult<HashMap<Category, FuzzySet>> {
        let mut results: HashMap<Category, FuzzySetBuilder> = HashMap::new();
        for rule in self.rules.iter() {
            let (out_category, out_term, y) = rule.apply(&self, values)?;
            // Get builder of outputs set or create new one using base set.
            let mut builder = results.remove(&out_category)
                .or(self.outputs.get(&out_category).map(|base_set| FuzzySetBuilder::new(base_set)))
                .ok_or(FuzzyError::InvalidCategory(out_category.clone()))?;
            // Apply threshold to givent term.
            builder.threshold(&out_term, y);
            // Put it bac in the Map.
            results.insert(out_category, builder);
        }
        println!("RES: {:?}", results);
        results.into_iter()
            .map(|(k, v)| v.build("out").map(|set| (k, set)))
            .collect()
    }

    /// Adds new rule to the Fuzzer.
    pub fn rule(
        mut self,
        rule: FuzzyRule
    ) -> Self {
        self.rules.push(rule);
        self
    }

    /// Adds new input set to the Fuzzer.
    pub fn fuzzify(
        mut self,
        ident: impl Into<Category>,
        category: FuzzySet
    ) -> Self {
        self.categories.insert(ident.into(), category);
        self
    }

    /// Adds new output set to the Fuzzer.
    pub fn defuzzify(
        mut self,
        ident: impl Into<Category>,
        output: FuzzySet
    ) -> Self {
        self.outputs.insert(ident.into(), output);
        self
    }

    fn call(
        &self,
        point: &FuzzyValue
    ) -> FuzzyResult<f64> {
        let (category, term, x) = point;
        self.categories.get(category)
            .ok_or(FuzzyError::InvalidCategory(category.clone()))?
            .call(*x)?
            .remove(term)
            .ok_or(FuzzyError::InvalidTerm(term.clone()))
    }
}

#[derive(Debug)]
struct FuzzySetBuilder<'a> {
    base: &'a FuzzySet,
    values: HashMap<Term, f64>
}

impl<'a> FuzzySetBuilder<'a> {
    fn new(
        base: &'a FuzzySet
    ) -> Self {
        Self {
            base,
            values: HashMap::new()
        }
    }

    fn threshold(
        &mut self,
        term: impl Into<Term>,
        y: f64
    ) -> &mut Self {
        let key = term.into();
        // Accumulation method should be configurable.
        // By default it takes MAX.
        let accum = |old: f64, new: f64| if new > old { new } else { old };
        let next = self.values.get(&key).map(|current| accum(*current, y)).unwrap_or_else(|| y);
        self.values.insert(key, next);
        self
    }

    fn build(
        &self,
        term_name: impl Into<Term>
    ) -> FuzzyResult<FuzzySet> {
        let mut set = self.base.clone();

        println!("SET1: {:?}", self);
        // Apply thresholds stored in self.values to output set
        for (term, _) in self.base.terms() {
            let thres = self.values.get(term).unwrap_or_else(|| &0.0);
            println!("{}: {}", term, thres);
            println!("BEFORE: {:?}", set);
            set.apply_threshold(term, *thres)?;
            println!("AFTER: {:?}", set);
        }

        // Calculate output set by evaluating xs of base output set.
        let mut xs = self.base.terms()
            .map(|(_, points)| points.iter().map(|(x, _)| *x).collect::<Vec<f64>>())
            .flatten()
            .collect::<Vec<f64>>();

        xs.sort_by(|x, y| x.partial_cmp(y).unwrap());
        xs.dedup();

        let mut points = Vec::with_capacity(xs.len());
        for x in xs.iter() {
            let results = set.call(*x)?;
            let y = results.into_iter().fold(0.0, |acc, (_, y)| acc+y);
            points.push((*x, y));
        }

        FuzzySet::new().term(term_name.into(), points)
    }
}

#[derive(Debug, PartialEq)]
pub enum FuzzyRule {
    Unit(FuzzyIdent, FuzzyIdent),
    And(Vec<FuzzyIdent>, FuzzyIdent),
    Or(Vec<FuzzyIdent>, FuzzyIdent)
}

impl FuzzyRule {
    pub fn apply(
        &self,
        fuzzer: &Fuzzer,
        values: &HashMap<Category, f64>
    ) -> FuzzyResult<FuzzyValue> {
        // Extract fuzzy sets identifiers and output category
        let (idents, (out_category, out_term)) = match self {
            FuzzyRule::Unit(ident, out) => (vec![ident.clone()], out.clone()),
            FuzzyRule::And(idents, out) => (idents.clone(), out.clone()),
            FuzzyRule::Or(idents, out) => (idents.clone(), out.clone())
        };

        // Sample fuzzy sets
        let mut ys = Vec::new();
        for (cat, term) in idents {
            let y = values.get(&cat)
                .ok_or(FuzzyError::InvalidCategory(cat.clone()))
                .and_then(|x| fuzzer.call(&(cat, term.clone(), *x)))?;
            ys.push(y);
        };

        // Pick return value based on samples.
        let cmp = |x: &f64, y: &f64| x.partial_cmp(y).unwrap();
        let mut ys = ys.into_iter();
        let y = match self {
            FuzzyRule::Unit(_, _) => ys.next(),
            FuzzyRule::And(_, _) => ys.min_by(cmp),
            FuzzyRule::Or(_, _) => ys.max_by(cmp),
        }.unwrap();
        Ok((out_category, out_term, y))
    }
}


#[macro_export]
macro_rules! unit {
    ($c1:expr=>$t1:expr; $co:expr=>$to:expr) => {
        $crate::FuzzyRule::Unit(
            (String::from($c1), String::from($t1)),
            (String::from($co), String::from($to)))
    }
}

// Macro generator for macors.
macro_rules! make_rule {
    ($d: tt $name:ident, $typ: tt) => {
        #[macro_export]
        #[allow(unused_macros)]
        macro_rules! $name {
            ($d($d c1:expr=>$d t1:expr),*;$co:expr=>$to:expr) => {
                $crate::FuzzyRule::$typ(
                    vec![$d((String::from($d c1), String::from($d t1)),)*],
                    (String::from($co), String::from($to))
                )
            }
        }
    }
}

make_rule!($ and, And);
make_rule!($ or, Or);

#[test]
fn test_macros(
) -> () {
    assert_eq!(
        unit!("loudness" => "quiet"; "change" => "keep"),
        FuzzyRule::Unit(
            ("loudness".to_string(), "quiet".to_string()),
            ("change".to_string(), "keep".to_string())
        )
    );
    assert_eq!(
        and!("loudness" => "quiet", "tod" => "morning", "param1" => "param2"; "change" => "vol up"),
        FuzzyRule::And(
            vec![
                ("loudness".to_string(), "quiet".to_string()),
                ("tod".to_string(), "morning".to_string()),
                ("param1".to_string(), "param2".to_string())
            ],
            ("change".to_string(), "vol up".to_string()))
    );
    assert_eq!(
        or!("loudness" => "quiet", "tod" => "morning", "param1" => "param2"; "change" => "vol up"),
        FuzzyRule::Or(
            vec![
                ("loudness".to_string(), "quiet".to_string()),
                ("tod".to_string(), "morning".to_string()),
                ("param1".to_string(), "param2".to_string())
            ],
            ("change".to_string(), "vol up".to_string()))
    );
    assert_eq!(
        and!("param1" => "param2"; "change" => "vol up"),
        FuzzyRule::And(
            vec![
                ("param1".to_string(), "param2".to_string())
            ],
            ("change".to_string(), "vol up".to_string()))
    );
}
