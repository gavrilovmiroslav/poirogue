use std::collections::HashSet;
use std::fmt::{Debug, Display, Formatter};
use petgraph::Graph;
use petgraph::dot::{Dot, Config};
use petgraph::graph::NodeIndex;
use rand::{Rng, thread_rng};
use rand::seq::SliceRandom;
use urlencoding::encode;

pub enum Person {
    Player,
    Victim,
    Suspect(u8),
}

impl Person {
    pub fn get_name(&self) -> String {
        random_names::RandomName::new().name
    }
}

impl Debug for Person {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_name())
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Relation {
    Killed,
    Absolves,
    Protects,
    Dislikes,
}

type MurderCase = Graph<Person, Relation>;

pub fn generate_murder() -> MurderCase {
    let mut rng = thread_rng();
    let mut case: MurderCase = Default::default();

    let victim_index = case.add_node(Person::Victim);
    let mut suspect_indices = (0..7)
        .map(|i| case.add_node(Person::Suspect(i)))
        .collect::<Vec<NodeIndex>>();

    let killer = rng.gen_range(0..7);
    let killer_index = suspect_indices[killer];

    let mut suspects_sans_killer = suspect_indices.clone();
    suspects_sans_killer.remove(killer);

    case.add_edge(killer_index,victim_index,Relation::Killed);

    let mut alibi_indices = suspect_indices.clone();

    fn connect(case: &mut MurderCase, fso: &Vec<NodeIndex>, sso: &Vec<NodeIndex>, mut pred: impl FnMut(&NodeIndex, &NodeIndex) -> bool, rel: Relation) {
        let mut rng = thread_rng();
        let mut fs = fso.clone();
        let mut ss = sso.clone();
        fs.shuffle(&mut rng);
        ss.shuffle(&mut rng);

        fs.iter().zip(&ss).for_each(|(a, b)| {
            if pred(a, b) {
                case.add_edge(*a, *b, rel);
            }
        });
    }

    connect(&mut case, &suspect_indices, &alibi_indices, |a, b| { *b != killer_index }, Relation::Absolves);
    connect(&mut case, &suspect_indices, &suspect_indices, |a, b| a != b && rng.gen_range(0..10) > 3, Relation::Protects);

    suspect_indices.push(victim_index);
    connect(&mut case, &alibi_indices, &suspect_indices, |a, b| a != b && rng.gen_range(0..10) > 4, Relation::Dislikes);

    suspects_sans_killer.shuffle(&mut rng);
    for i in suspects_sans_killer.iter().take(rng.gen_range(3..5)) {
        case.add_edge(killer_index, *i, Relation::Dislikes);
    }

    let dot = Dot::new(&case);
    let encoded_dot = format!("{}", encode(&*format!("{:?}", dot)));

    open::that(format!("https://dreampuf.github.io/GraphvizOnline/#{}", encoded_dot)).unwrap();
    case
}
