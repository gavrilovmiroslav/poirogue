#pragma once

#include <cstdio>
#include <limits>
#include <queue>
#include <string>
#include <tuple>
#include <unordered_set>
#include <vector>
#include "engine.h"

namespace metric
{
	struct Any {};

	struct Distance {};
}

namespace graphs 
{
	using NodeEntity = Entity;

	using EdgeEntity = Entity;
	using WeightedEdgeEntity = std::tuple<EdgeEntity, float>;

	struct Node {
		std::vector<Entity> edges;
	};

	struct Arrow
	{
		NodeEntity source;
		NodeEntity target;
		bool mutual;
	};

	struct Label
	{
		std::string label;
	};

	template<typename P = metric::Any>
	struct Weight
	{
		float weight;
	};

	struct Graph
	{
	private:
		ecs::registry contents;
		bool new_node_created;
	public:

		// CREATION

		NodeEntity create_node()
		{
			auto node = contents.create();
			contents.emplace_or_replace<Node>(node);
			new_node_created = true;
			return node;
		}

		EdgeEntity create_arrow(NodeEntity a, NodeEntity b)
		{
			auto edge = contents.create();

			assert(contents.all_of<Node>(a));
			auto& node_a = contents.get<Node>(a);
			node_a.edges.push_back(edge);

			contents.emplace_or_replace<Arrow>(edge, a, b, false);
			return edge;
		}

		NodeEntity get_source(EdgeEntity a)
		{
			assert(contents.all_of<Arrow>(a));
			return contents.get<Arrow>(a).source;
		}

		NodeEntity get_target(EdgeEntity a)
		{
			assert(contents.all_of<Arrow>(a));
			return contents.get<Arrow>(a).target;
		}

		NodeEntity get_target_from(EdgeEntity edge, NodeEntity side)
		{
			assert(contents.all_of<Arrow>(edge));
			auto& arrow = contents.get<Arrow>(edge);
			assert(arrow.source == side || (arrow.mutual && arrow.target == side));

			if (arrow.source == side)
				return arrow.target;
			else
				return arrow.source;
		}

		EdgeEntity create_arrows(NodeEntity a, NodeEntity b)
		{
			auto edge = contents.create();
			assert(contents.all_of<Node>(a));
			auto& node_a = contents.get<Node>(a);
			node_a.edges.push_back(edge);

			assert(contents.all_of<Node>(b));
			auto& node_b = contents.get<Node>(b);
			node_b.edges.push_back(edge);

			contents.emplace_or_replace<Arrow>(edge, a, b, true);
			return edge;
		}

		// LABELING

		void label_node(NodeEntity node, std::string label)
		{
			assert(contents.all_of<Node>(node));
			contents.emplace_or_replace<Label>(node, label);
		}
		
		std::string get_node_label(NodeEntity node) const noexcept
		{
			if (contents.all_of<Node, Label>(node))
				return contents.get<Label>(node).label;
			else
				return "";
		}

		void label_edge(EdgeEntity edge, std::string label)
		{
			assert(contents.all_of<Arrow>(edge));
			contents.emplace_or_replace<Label>(edge, label);
		}

		std::string get_edge_label(EdgeEntity edge) const noexcept
		{
			if (contents.all_of<Arrow, Label>(edge))
				return contents.get<Label>(edge).label;
			else
				return "";
		}

		// WEIGHING

		template<typename W = metric::Any>
		void weigh_node(NodeEntity node, float weight)
		{
			assert(contents.all_of<Node>(node));
			contents.emplace_or_replace<Weight<W>>(node, weight);
		}

		template<typename W = metric::Any>
		float get_node_weight(NodeEntity node)
		{
			if (contents.all_of<Node, Weight<W>>(node))
				return contents.get<Weight<W>>(node).weight;
			else
				return std::numeric_limits<float>::infinity();
		}

		template<typename W = metric::Any>
		void weigh_edge(EdgeEntity edge, float weight)
		{
			assert(contents.all_of<Arrow>(edge));
			contents.emplace_or_replace<Weight<W>>(edge, weight);
		}

		template<typename W = metric::Any>
		float get_edge_weight(EdgeEntity edge)
		{
			if (contents.all_of<Arrow, Weight<W>>(edge))
				return contents.get<Weight<W>>(edge).weight;
			else
				return std::numeric_limits<float>::infinity();
		}

		// TAGGING

		template<typename Tag, typename... Args>
		void tag_node(NodeEntity node, Args&&... args)
		{
			assert(contents.all_of<Node>(node));
			contents.emplace_or_replace<Tag>(node, args...);			
		}

		template<typename Tag>
		Tag& get_tag(NodeEntity node)
		{			
			return contents.get<Tag>(node);
		}

		template<typename Tag, typename... Args>
		void tag_edge(EdgeEntity edge, Args&&... args)
		{
			assert(contents.all_of<Arrow>(edge));
			contents.emplace_or_replace<Tag>(edge, args...);
		}

		// ACCESSORS

		std::vector<NodeEntity> get_all_nodes()
		{
			static std::vector<NodeEntity> nodes;
			if (new_node_created)
			{
				nodes.clear();
				for (auto&& [entity, _] : contents.view<Node>().each())
				{
					nodes.push_back(entity);
				}
			}

			return nodes;
		}

		std::vector<EdgeEntity> get_all_edges(NodeEntity node)
		{
			assert(contents.all_of<Node>(node));
			return contents.get<Node>(node).edges;
		}

		template<typename W = metric::Any>
		std::vector<WeightedEdgeEntity> get_all_weighted_edges_from_node(NodeEntity node)
		{
			std::vector<WeightedEdgeEntity> edges;
			assert(contents.all_of<Node>(node));
			for (auto edge : contents.get<Node>(node).edges)
			{
				if (contents.all_of<Arrow, Weight<W>>(edge))
				{
					edges.push_back({ edge, contents.get<Weight<W>>(edge).weight });
				}
			}

			return edges;
		}

		std::vector<NodeEntity> get_all_neighbors(NodeEntity node)
		{
			std::vector<Entity> neighbors;

			assert(contents.all_of<Node>(node));
			for (auto edge : contents.get<Node>(node).edges)
			{
				assert(contents.all_of<Arrow>(edge));
				auto& arrow = contents.get<Arrow>(edge);
				if (arrow.source == node)
				{
					neighbors.push_back(arrow.target);
				} 
				else if (arrow.mutual && arrow.target == node)
				{
					neighbors.push_back(arrow.source);
				}
			}

			return neighbors;
		}

		// ALGORITHMS

		template<typename W = metric::Any>
		std::vector<EdgeEntity> get_minimum_spanning_tree()
		{	
			constexpr const int NODE_COMP = 0;
			constexpr const int EDGE_COMP = 0;
			constexpr const int WEIGHT_COMP = 1;

			auto heap_cmp = [](WeightedEdgeEntity left, WeightedEdgeEntity right) { return std::get<WEIGHT_COMP>(left) > std::get<WEIGHT_COMP>(right); };
			using Heap = std::priority_queue<WeightedEdgeEntity, std::vector<WeightedEdgeEntity>, decltype(heap_cmp)>;

			std::vector<EdgeEntity> edges;
			std::unordered_set<EdgeEntity> taken;
			std::unordered_set<NodeEntity> visited;
			
			auto nodes = get_all_nodes();
			if (nodes.size() == 0) return edges;

			Heap freelist(heap_cmp);
			{
				auto weighted_from_source = get_all_weighted_edges_from_node(nodes[0]);
				std::sort(weighted_from_source.begin(), weighted_from_source.end(), heap_cmp);
				auto first = weighted_from_source[weighted_from_source.size() - 1];
				freelist.push(first);
				visited.insert(get_source(std::get<EDGE_COMP>(first)));
			}

			while (!freelist.empty())
			{
				auto top_free_weighted_edge = freelist.top();
				auto edge = std::get<EDGE_COMP>(top_free_weighted_edge);
				auto weight = std::get<WEIGHT_COMP>(top_free_weighted_edge);

				freelist.pop();
				edges.push_back(edge);
				taken.insert(edge);
				
				auto far_end = (visited.count(get_source(edge)) > 0) ? get_target(edge) : get_source(edge);				
				visited.insert(far_end);

				auto weighted_from_far_end = get_all_weighted_edges_from_node(far_end);
				std::sort(weighted_from_far_end.begin(), weighted_from_far_end.end(), heap_cmp);
				std::reverse(weighted_from_far_end.begin(), weighted_from_far_end.end());
				for (auto weighted_edge : weighted_from_far_end)
				{
					auto next_edge = std::get<EDGE_COMP>(weighted_edge);
					auto next_weight = std::get<WEIGHT_COMP>(weighted_edge);
					if (taken.count(next_edge) > 0) continue;

					auto next_far_end = get_target_from(next_edge, far_end);
					if (visited.count(next_far_end) > 0) continue;

					freelist.push(weighted_edge);
					break;
				}
			}

			return edges;
		}

		// PRINTER

		void print()
		{
			printf("digraph G {\n");
			for (auto&& [entity, node] : contents.view<Node>().each())
			{
				auto label = get_node_label(entity);
				printf("\tv%d [label=\"%s\"];\n", entity, label.c_str());
			}

			for (auto&& [entity, edge] : contents.view<Arrow>().each())
			{
				auto label = get_edge_label(entity);
				auto source = get_source(entity);
				auto target = get_target(entity);
				printf("\tv%d -> v%d [label=\"%s\"];\n", source, target, label.c_str());
			}
			printf("}\n");
		}
	};
}