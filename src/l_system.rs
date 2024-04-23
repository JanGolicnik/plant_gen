use std::collections::HashMap;

use rand::Rng;

pub struct LSystem {
    symbols: Vec<char>,
}

impl LSystem {
    pub fn new(symbols: String, iterations: u32) -> Self {
        struct Rule {
            result: &'static str,
            probability: f32,
        }

        let rule = |result, probability| Rule {
            result,
            probability,
        };

        let rules = HashMap::from([
            (
                'F',
                vec![rule("F", 0.1), rule("FF", 0.85), rule("FFF", 0.05)],
            ),
            (
                'X',
                vec![
                    rule("F[+X][-X]FX", 0.5),
                    rule("F[+X]FX", 0.05),
                    rule("F[-X]FX", 0.05),
                    rule("F[++X][-X]FX", 0.1),
                    rule("F[+X][--X]FX", 0.1),
                    rule("F[+X][-X]FA", 0.2),
                ],
            ),
        ]);

        let mut rng = rand::thread_rng();

        let mut pick_rule = |rules: &[Rule]| {
            let n = rng.gen::<f32>();
            let mut t = 0.0;
            for rule in rules.iter() {
                t += rule.probability;
                if t > n {
                    return Some(rule.result);
                }
            }
            None
        };

        let mut symbols = symbols.chars().collect::<Vec<_>>();
        (0..iterations).for_each(|_| {
            let mut new_symbols = Vec::new();
            for symbol in symbols.iter() {
                if let Some(rules) = rules.get(symbol) {
                    let mut new_chars = pick_rule(rules).unwrap().chars().collect::<Vec<_>>();
                    new_symbols.append(&mut new_chars);
                } else {
                    new_symbols.push(*symbol);
                }
            }
            symbols = std::mem::take(&mut new_symbols);
        });
        Self { symbols }
    }

    pub fn symbols(&self) -> std::slice::Iter<'_, char> {
        self.symbols.iter()
    }
}
