use std::collections::HashSet;
use std::fmt::{Debug, Display, Formatter};
use petgraph::Graph;
use petgraph::dot::{Dot, Config};
use petgraph::graph::NodeIndex;
use rand::{Rng, thread_rng};
use rand::rngs::ThreadRng;
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
pub enum PosOpinion {
    Loves,
    Likes,
    Adores,
    Respects,
}

#[derive(Debug, Copy, Clone)]
pub enum NegOpinion {
    Hates,
    Dislikes,
    BoredWith,
    Disdains
}

#[derive(Debug, Copy, Clone)]
pub enum Opinion {
    Good(PosOpinion),
    Neutral,
    Bad(NegOpinion)
}

#[derive(Debug, Copy, Clone)]
pub enum Relation {
    Killed,
    Absolves,
    Protects,
    Envies,
    Past(Opinion)
}

fn random_gossip(rng: &mut ThreadRng) -> Relation {
    match rng.gen_range(1..10) {
        1 => Relation::Past(Opinion::Good(PosOpinion::Loves)),
        2 => Relation::Past(Opinion::Good(PosOpinion::Likes)),
        3 => Relation::Past(Opinion::Good(PosOpinion::Adores)),
        4 => Relation::Past(Opinion::Good(PosOpinion::Respects)),
        5 => Relation::Past(Opinion::Good(PosOpinion::Respects)),
        6 => Relation::Past(Opinion::Bad(NegOpinion::Hates)),
        7 => Relation::Past(Opinion::Bad(NegOpinion::Dislikes)),
        8 => Relation::Past(Opinion::Bad(NegOpinion::BoredWith)),
        9 => Relation::Past(Opinion::Bad(NegOpinion::Disdains)),
        10 => Relation::Past(Opinion::Bad(NegOpinion::Disdains)),
        _ => Relation::Past(Opinion::Neutral)
    }
}

fn nice_opinion_names(mut dot_code: String) -> String {
    dot_code = dot_code.replace("Past(Good(Loves))\"", "Loved\", color = \"pink;0.75:grey\", arrowhead=\"dot\", fontsize=12, style=\"dashed\"");
    dot_code = dot_code.replace("Past(Good(Adores))\"", "Adored\", color = \"pink;0.75:grey\", arrowhead=\"dot\", fontsize=12, style=\"dashed\"");
    dot_code = dot_code.replace("Past(Good(Likes))\"", "Liked\", color = \"pink;0.75:grey\", arrowhead=\"dot\", fontsize=12, style=\"dashed\"");
    dot_code = dot_code.replace("Past(Good(Respects))\"", "Respected\", color = \"pink;0.75:grey\", arrowhead=\"dot\", fontsize=12, style=\"dashed\"");
    dot_code = dot_code.replace("Past(Bad(Hates))\"", "Hated\", color = \"orange;0.35:grey\", arrowhead=\"box\", fontsize=12, style=\"dashed\"");
    dot_code = dot_code.replace("Past(Bad(BoredWith))\"", "Bored With\", color = \"orange;0.35:grey\", arrowhead=\"box\", fontsize=12, style=\"dashed\"");
    dot_code = dot_code.replace("Past(Bad(Disdains))\"", "Disdained\", color = \"orange;0.35:grey\", arrowhead=\"box\", fontsize=12, style=\"dashed\"");
    dot_code = dot_code.replace("Past(Bad(Dislikes))\"", "Disliked\", color = \"orange;0.35:grey\", arrowhead=\"box\", fontsize=12, style=\"dashed\"");
    dot_code
}

fn colorize(mut dot_code: String) -> String {
    dot_code = dot_code.replace("Killed\"", "Killed\", color = \"crimson\", penwidth = 2");
    dot_code = dot_code.replace("Absolves\"", "Absolves\", color = \"cornflowerblue\", penwidth = 2");
    dot_code = dot_code.replace("Protects\"", "Protects\", color = \"blueviolet\"");
    dot_code = dot_code.replace("Envies\"", "Envies\", color = \"chartreuse3\"");
    dot_code
}

type MurderCase = Graph<Person, Relation>;

pub fn generate_murder() -> MurderCase {
    let mut rng = thread_rng();
    let mut case_graph: MurderCase = Default::default();

    let victim_index = case_graph.add_node(Person::Victim);
    let mut suspect_indices = (0..7)
        .map(|i| case_graph.add_node(Person::Suspect(i)))
        .collect::<Vec<NodeIndex>>();

    let killer = rng.gen_range(0..7);
    let killer_index = suspect_indices[killer];

    let mut suspects_sans_killer = suspect_indices.clone();
    suspects_sans_killer.remove(killer);

    case_graph.add_edge(killer_index, victim_index, Relation::Killed);

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

    connect(&mut case_graph, &suspect_indices, &alibi_indices, |a, b| { *b != killer_index }, Relation::Absolves);
    connect(&mut case_graph, &suspect_indices, &suspect_indices, |a, b| a != b && rng.gen_range(0..10) > 3, Relation::Protects);

    for _i in 1..5 {
        let gossip = random_gossip(&mut rng);
        connect(&mut case_graph, &suspect_indices, &suspect_indices, |a, b| a != b && rng.gen_range(0..10) > 7, gossip);
    }

    suspect_indices.push(victim_index);
    connect(&mut case_graph, &alibi_indices, &suspect_indices, |a, b| a != b && rng.gen_range(0..10) > 6, Relation::Envies);
    connect(&mut case_graph, &alibi_indices, &suspect_indices, |a, b| a != b && rng.gen_range(0..10) > 6, Relation::Envies);

    suspects_sans_killer.shuffle(&mut rng);
    for i in suspects_sans_killer.iter().take(rng.gen_range(1..3)) {
        case_graph.add_edge(killer_index, *i, Relation::Envies);
    }

    suspect_indices.shuffle(&mut rng);
    for i in suspect_indices.iter().take(rng.gen_range(5..10)) {
        if *i == victim_index { continue }
        case_graph.add_edge(*i, victim_index, random_gossip(&mut rng));
    }

    let dot = Dot::new(&case_graph);
    let mut dot_code = String::from(&*format!("{:?}", dot));
    let encoded_dot = format!("https://dreampuf.github.io/GraphvizOnline/#{}",
                              encode(nice_opinion_names(colorize(dot_code)).as_str()));

    open::that(encoded_dot).unwrap();
    case_graph
}
