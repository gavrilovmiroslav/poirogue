use std::collections::hash_map::RandomState;
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;
use petgraph::Graph;
use petgraph::dot::{Dot, Config};
use petgraph::graph::NodeIndex;
use petgraph::prelude::EdgeRef;
use rand::{Rng, thread_rng};
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use urlencoding::encode;
use crate::VirtualKeyCode::P;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Person {
    Killer,
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

impl Display for Person {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_name())
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum PosOpinion {
    Loves,
    Likes,
    Adores,
    Respects,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum NegOpinion {
    Hates,
    Dislikes,
    Disdains,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Opinion {
    Good(PosOpinion),
    Neutral,
    Bad(NegOpinion)
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Emotion {
    Pity,
    Envy,
    Anger,
    Revulsion,
    Fear,
    Lust,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Relation {
    Killed,
    Absolves,
    Protects,
    Beholden,
    Feels(Emotion),
    Gossip(Opinion)
}

impl Display for Relation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

fn random_emotion(rng: &mut ThreadRng) -> Emotion {
    match rng.gen_range(1..6) {
        1 => Emotion::Anger,
        2 => Emotion::Envy,
        3 => Emotion::Fear,
        4 => Emotion::Revulsion,
        5 => Emotion::Pity,
        _ => Emotion::Lust
    }
}

fn random_gossip(rng: &mut ThreadRng) -> Relation {
    match rng.gen_range(1..10) {
        1 => Relation::Gossip(Opinion::Good(PosOpinion::Loves)),
        2 => Relation::Gossip(Opinion::Good(PosOpinion::Likes)),
        3 => Relation::Gossip(Opinion::Good(PosOpinion::Adores)),
        4..=5 => Relation::Gossip(Opinion::Good(PosOpinion::Respects)),
        6 => Relation::Gossip(Opinion::Bad(NegOpinion::Hates)),
        7..=8 => Relation::Gossip(Opinion::Bad(NegOpinion::Dislikes)),
        9 => Relation::Gossip(Opinion::Bad(NegOpinion::Disdains)),
        _ => Relation::Gossip(Opinion::Neutral)
    }
}

fn random_strong_gossip(rng: &mut ThreadRng) -> Relation {
    match rng.gen_bool(0.5) {
        true => Relation::Gossip(Opinion::Good(PosOpinion::Loves)),
        false => Relation::Gossip(Opinion::Bad(NegOpinion::Hates)),
    }
}

fn random_mostly_negative_gossip(rng: &mut ThreadRng) -> Relation {
    match rng.gen_range(1..20) {
        1 => Relation::Gossip(Opinion::Good(PosOpinion::Likes)),
        2 => Relation::Gossip(Opinion::Good(PosOpinion::Loves)),
        3 => Relation::Gossip(Opinion::Good(PosOpinion::Adores)),
        4 => Relation::Gossip(Opinion::Good(PosOpinion::Respects)),
        5..=12 => Relation::Gossip(Opinion::Bad(NegOpinion::Dislikes)),
        13..=17 => Relation::Gossip(Opinion::Bad(NegOpinion::Disdains)),
        18..=20 => Relation::Gossip(Opinion::Bad(NegOpinion::Hates)),
        _ => Relation::Gossip(Opinion::Neutral)
    }
}

fn nice_opinion_names(mut dot_code: String) -> String {
    dot_code = dot_code.replace("Gossip(Good(Loves))\"", "Loves\", color = \"deeppink3\", arrowhead=\"dot\", fontsize=12, style=\"dashed\"");
    dot_code = dot_code.replace("Gossip(Good(Adores))\"", "Adores\", color = \"deeppink3\", arrowhead=\"dot\", fontsize=12, style=\"dashed\"");
    dot_code = dot_code.replace("Gossip(Good(Likes))\"", "Likes\", color = \"deeppink3\", arrowhead=\"dot\", fontsize=12, style=\"dashed\"");
    dot_code = dot_code.replace("Gossip(Good(Respects))\"", "Respects\", color = \"deeppink3\", arrowhead=\"dot\", fontsize=12, style=\"dashed\"");
    dot_code = dot_code.replace("Gossip(Bad(Hates))\"", "Hates\", color = \"orange\", arrowhead=\"box\", fontsize=12, style=\"dashed\"");
    dot_code = dot_code.replace("Gossip(Bad(Disdains))\"", "Disdains\", color = \"orange\", arrowhead=\"box\", fontsize=12, style=\"dashed\"");
    dot_code = dot_code.replace("Gossip(Bad(Dislikes))\"", "Dislikes\", color = \"orange\", arrowhead=\"box\", fontsize=12, style=\"dashed\"");
    dot_code
}

fn colorize(mut dot_code: String) -> String {
    dot_code = dot_code.replace("Killed\"", "Killed\", color = \"crimson\", penwidth = 2");
    dot_code = dot_code.replace("Absolves\"", "Absolves\", color = \"cornflowerblue\", penwidth = 2");
    dot_code = dot_code.replace("Beholden\"", "Beholden\", color = \"green\"");
    dot_code = dot_code.replace("Protects\"", "Protects\", color = \"blueviolet\"");
    dot_code = dot_code.replace("Feels(Envy)\"", "Envies\", color = \"chartreuse3\"");
    dot_code = dot_code.replace("Feels(Pity)\"", "Pities\", color = \"goldenrod4\"");
    dot_code = dot_code.replace("Feels(Anger)\"", "Angry with\", color = \"coral1\"");
    dot_code = dot_code.replace("Feels(Revulsion)\"", "Revulsed by\", color = \"cadetblue4\"");
    dot_code = dot_code.replace("Feels(Fear)\"", "Fears\", color = \"darkgoldenrod3\"");
    dot_code
}

type MurderCase = Graph<Person, Relation>;
type MurderRelations = HashMap<Person, (Person, Relation)>;

pub fn generate_murder() -> (MurderCase, MurderRelations) {
    let mut rng = thread_rng();
    let mut case_graph: MurderCase = Default::default();

    let victim_index = case_graph.add_node(Person::Victim);
    let killer_index = case_graph.add_node(Person::Killer);

    let mut suspect_indices = (0..7)
        .map(|i| case_graph.add_node(Person::Suspect(i)))
        .collect::<Vec<NodeIndex>>();

    let mut suspects_sans_killer = suspect_indices.clone();

    suspect_indices.push(killer_index);
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
    connect(&mut case_graph, &suspect_indices, &alibi_indices, |a, b| a != b && rng.gen_range(0..10) > 6, Relation::Beholden);
    connect(&mut case_graph, &suspect_indices, &suspect_indices, |a, b| a != b && rng.gen_range(0..10) > 3, Relation::Protects);

    for _i in 1..5 {
        connect(&mut case_graph, &suspect_indices, &suspect_indices, |a, b| a != b && rng.gen_range(0..10) > 7, random_gossip(&mut thread_rng()));
    }

    suspect_indices.push(victim_index);
    connect(&mut case_graph, &alibi_indices, &suspect_indices, |a, b| a != b && rng.gen_range(0..10) > 6, Relation::Feels(random_emotion(&mut thread_rng())));

    suspects_sans_killer.shuffle(&mut rng);
    for i in suspects_sans_killer.iter().take(rng.gen_range(1..3)) {
        case_graph.add_edge(killer_index, *i, Relation::Feels(random_emotion(&mut rng)));
    }

    suspect_indices.shuffle(&mut rng);
    for i in suspect_indices.iter().take(rng.gen_range(5..7)) {
        if *i == victim_index { continue }
        case_graph.add_edge(*i, victim_index, random_mostly_negative_gossip(&mut rng));
    }

    suspect_indices.shuffle(&mut rng);
    for i in suspect_indices.iter().take(rng.gen_range(2..4)) {
        if *i == victim_index { continue }
        case_graph.add_edge(victim_index, *i, random_strong_gossip(&mut rng));
    }

    let murder_edges = case_graph.edges_connecting(killer_index, victim_index);
    let murder_edge_weights: MurderRelations = {
        let mut edges: MurderRelations = HashMap::new();
        murder_edges.for_each(|edge| {
            let source = *case_graph.node_weight(edge.source()).unwrap();
            let target = *case_graph.node_weight(edge.target()).unwrap();
            let weight = *edge.weight();
            edges.insert(source, (target, weight));
        });

        edges
    };

    let dot = Dot::new(&case_graph);
    let mut dot_code = String::from(&*format!("{:?}", dot));
    let encoded_dot = format!("https://dreampuf.github.io/GraphvizOnline/#{}",
                              encode(nice_opinion_names(colorize(dot_code)).as_str()));

    open::that(encoded_dot).unwrap();
    (case_graph, murder_edge_weights)
}
