#pragma once

#include "common.h"

void tell_buddy_about_event(PeopleMapping& mapping, Entity who, Entity what_event, int time, int max_count = 100)
{
	TCODRandom* rng = TCODRandom::getInstance();

	auto all_visiting = mapping.get_all_visiting_with(who);
	shuffle(all_visiting);

	auto talk_event = mapping.graph->create_node();
	mapping.graph->label_node(talk_event, "RANDOM TALK");

	auto talk_time = time + rng->getInt(1, time / 2);
	if (talk_time > 0) talk_time = -1;
	mapping.graph->tag_node<Time>(talk_event, talk_time);

	mapping.graph->label_edge(mapping.graph->create_arrow(who, talk_event), "talked");
	for (int i = 0; i < std::min(max_count, rng->getInt(1, all_visiting.size())); i++)
	{
		mapping.graph->label_edge(mapping.graph->create_arrow(all_visiting[i], talk_event), "listened");
	}

	mapping.graph->label_edge(mapping.graph->create_arrow(talk_event, what_event), "about");
}

void tell_work_buddy_about_event(PeopleMapping& mapping, Entity who, Entity what_event, int time, int max_count = 100)
{
	TCODRandom* rng = TCODRandom::getInstance();

	auto all_workers = mapping.get_all_working_with(who);
	shuffle(all_workers);

	auto talk_event = mapping.graph->create_node();
	mapping.graph->label_node(talk_event, "TALK AT WORK");
	
	auto talk_time = time + rng->getInt(1, time / 2);
	if (talk_time > 0) talk_time = -1;
	mapping.graph->tag_node<Time>(talk_event, talk_time);

	mapping.graph->label_edge(mapping.graph->create_arrow(who, talk_event), "talked");
	for (int i = 0; i < std::min(max_count, rng->getInt(1, all_workers.size())); i++)
	{
		mapping.graph->label_edge(mapping.graph->create_arrow(all_workers[i], talk_event), "listened");
	}

	mapping.graph->label_edge(mapping.graph->create_arrow(talk_event, what_event), "about");
}

void murder(PeopleMapping& mapping, Entity killer)
{
	auto murder = mapping.graph->create_arrow(killer, mapping.get_victim());
	mapping.graph->label_edge(murder, "murder");
}

void murder_debt_scare(PeopleMapping& mapping, bool is_murder = true)
{
	TCODRandom* rng = TCODRandom::getInstance();

	// some past debt
	auto past_debt_event = mapping.graph->create_node();
	mapping.graph->label_node(past_debt_event, "PAST DEBT");
	mapping.graph->tag_node<Time>(past_debt_event, -100);

	auto active_debt = mapping.graph->create_node();
	mapping.graph->label_node(active_debt, "DEBT");
	auto active_debt_time = rng->getInt(-30, -7);
	mapping.graph->tag_node<Time>(active_debt, active_debt_time);

	auto people = mapping.get_all_people_shuffled();

	auto old_debtee = people.back();
	people.pop_back();

	auto rich_killer = people.back();
	people.pop_back();

	Entity victim = mapping.get_victim();

	if (is_murder)
		murder(mapping, rich_killer);
	else
	{
		victim = people.back();
		people.pop_back();
	}

	mapping.graph->label_edge(mapping.graph->create_arrow(victim, active_debt), "borrowed");
	mapping.graph->label_edge(mapping.graph->create_arrow(rich_killer, active_debt), "lend");

	mapping.graph->label_edge(mapping.graph->create_arrow(old_debtee, past_debt_event), "returned money");
	mapping.graph->label_edge(mapping.graph->create_arrow(rich_killer, past_debt_event), "threatened");

	auto economic_stability = mapping.graph->create_node();
	mapping.graph->label_node(economic_stability, "HAS MONEY");
	mapping.graph->label_edge(mapping.graph->create_arrow(economic_stability, victim), "who");

	tell_work_buddy_about_event(mapping, victim, economic_stability, rng->getInt(active_debt_time, -1));
	tell_buddy_about_event(mapping, victim, economic_stability, rng->getInt(active_debt_time, -1));

	auto short_scuffle = mapping.graph->create_node();
	mapping.graph->label_node(short_scuffle, "SHORT SCUFFLE");
	auto scuffle_time = rng->getInt(-3, -1);
	mapping.graph->tag_node<Time>(short_scuffle, scuffle_time);

	mapping.graph->label_edge(mapping.graph->create_arrow(short_scuffle, active_debt), "about");
	mapping.graph->label_edge(mapping.graph->create_arrow(rich_killer, short_scuffle), "threatened");
	mapping.graph->label_edge(mapping.graph->create_arrow(victim, short_scuffle), "was threatened");
	mapping.graph->label_edge(mapping.graph->create_arrow(old_debtee, short_scuffle), "seen");

	tell_buddy_about_event(mapping, old_debtee, short_scuffle, scuffle_time, 1);
}

void murder_old_grievance_revenge(PeopleMapping& mapping, bool is_murder = true)
{
	TCODRandom* rng = TCODRandom::getInstance();

	auto people = mapping.get_all_people_shuffled();
	auto killer = people.back();
	people.pop_back();

	Entity victim = mapping.get_victim();

	if (is_murder)
		murder(mapping, killer);
	else
	{
		victim = people.back();
		people.pop_back();
	}

	// some past event, war, love, whatever
	auto past_event = mapping.graph->create_node();
	mapping.graph->label_node(past_event, "HARSH PAST");
	mapping.graph->tag_node<Time>(past_event, -100);
	// call this to figure out what happened:
	// generate_past_event(past_event);
	
	// victim has a past
	auto victim_past = mapping.graph->create_arrow(victim, past_event);
	mapping.graph->label_edge(victim_past, "remembers (bully)");

	// the killer shares the same past
	mapping.graph->label_edge(mapping.graph->create_arrow(killer, past_event), "remembers (bullied)");
	tell_buddy_about_event(mapping, killer, past_event, -100, 2);

	// some recent violent event
	auto recent_violent_event = mapping.graph->create_node();
	auto recent_violent_time = rng->getInt(-14, -7);

	mapping.graph->label_node(recent_violent_event, "SHORT FIGHT!");
	mapping.graph->tag_node<Time>(recent_violent_event, recent_violent_time);

	auto places = mapping.get_all_places_shuffled();
	auto place = places.back(); places.pop_back();
	
	mapping.graph->label_edge(mapping.graph->create_arrow(recent_violent_event, place), "happened near");

	auto all_similar_visit = mapping.get_all_visiting_with(killer);
	Entity almost_victim = entt::null;

	if (all_similar_visit.size() > 0)
	{
		shuffle(all_similar_visit);
		almost_victim = all_similar_visit.back(); all_similar_visit.pop_back();
	}
	else
	{
		almost_victim = people.back(); people.pop_back();
	}

	mapping.graph->label_edge(mapping.graph->create_arrow(almost_victim, recent_violent_event), "was attacked");
	mapping.graph->label_edge(mapping.graph->create_arrow(killer, recent_violent_event), "mistakenly attacked");

	tell_work_buddy_about_event(mapping, almost_victim, recent_violent_event, recent_violent_time);
	tell_buddy_about_event(mapping, almost_victim, recent_violent_event, recent_violent_time, 2);

	auto lookalike = mapping.graph->create_node();
	mapping.graph->label_node(lookalike, "LOOK ALIKE");
	
	mapping.graph->label_edge(mapping.graph->create_arrow(almost_victim, lookalike), "looks like");
	mapping.graph->label_edge(mapping.graph->create_arrow(victim, lookalike), "looks like");

	mapping.graph->label_edge(mapping.graph->create_arrow(recent_violent_event, lookalike), "because");
}
