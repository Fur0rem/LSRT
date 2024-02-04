import osmnx as ox
import networkx as nx
import random

from CityGraph import CityGraph
from Attack import Attack, print_suppression_edge

# Juste pour test pour l'instant
ville="Saints"

def random_attack(graph : CityGraph, nbTemps : int, budget : int) -> Attack :
	"""Each time, it removes a random edge from the graph until the budget is reached or no more edges are available"""

	attacks = Attack(graph)
	edges = list(graph.edges())
	
	for time in range(nbTemps) :
		edges_t = edges.copy()
		budget_used = 0
		nb_retires = 0
		while budget_used < budget and len(edges_t) > 0 :
			edge_idx = random.randint(0, graph.nb_edges() - (nb_retires + 1))
			cost = graph.get_nb_lanes_of_edge(edge_idx)
			if cost <= budget - budget_used :
				attacks.add_attack(time, edge_idx)
				budget_used += cost
				nb_retires += 1
			edges_t.pop(edge_idx)

	return attacks

def sorted_attack(graph : CityGraph, nbTemps : int, budget : int, sort_by : callable) -> Attack :
	"""Each time, it removes the edge that is the most affected by the sort_by function until the budget is reached or no more edges are available"""

	attacks = Attack(graph)
	edges = list(graph.edges())
	for i, edge in enumerate(edges) :
		edges[i] = graph.find_edge_index(edge[0], edge[1])
	edges.sort(key = sort_by, reverse = True)

	for time in range(nbTemps) :
		edges_t = edges.copy()
		budget_used = 0
		nb_retires = 0
		while budget_used < budget and len(edges_t) > 0 :
			edge_idx = edges_t.pop(0)
			cost = graph.get_nb_lanes_of_edge(edge_idx)
			if cost <= budget - budget_used :
				attacks.add_attack(time, edge_idx)
				budget_used += cost
				nb_retires += 1
			edges_t.pop(edge_idx)

	return attacks

def min_lanes_attack(graph : CityGraph, nbTemps : int, budget : int) -> Attack :
	"""Each time, it removes the edge with the least lanes until the budget is reached or no more edges are available"""
	return sorted_attack(graph, nbTemps, budget, lambda x : -graph.get_nb_lanes_of_edge(x))

def max_lanes_attack(graph : CityGraph, nbTemps : int, budget : int) -> Attack :
	"""Each time, it removes the edge with the most lanes until the budget is reached or no more edges are available"""
	return sorted_attack(graph, nbTemps, budget, lambda x : graph.get_nb_lanes_of_edge(x))

def betweenness_centralities_attack(graph : CityGraph, nbTemps : int, budget : int) -> Attack :
	"""Each time, it removes the edge with the highest betweenness centrality until the budget is reached or no more edges are available"""
	return sorted_attack(graph, nbTemps, budget, lambda x : graph.get_edge_betweenness_centrality(x))

graph = CityGraph(ville)
graph.print_stats()
graph.show()

a = random_attack(graph, 50, 50)
a.write_to_file("test_random_attack.txt")
a = min_lanes_attack(graph, 10, 10)
a.write_to_file("test_min_lanes_attack.txt")
a = max_lanes_attack(graph, 10, 10)
a.write_to_file("test_max_lanes_attack.txt")
a = betweenness_centralities_attack(graph, 10, 10)
a.write_to_file("test_betweenness_centralities_attack.txt")