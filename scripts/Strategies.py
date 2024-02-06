import random

from CityGraph import CityGraph
from Attack import Attack, print_suppression_edge

# Juste pour test pour l'instant
ville="Saints"

def random_attack(graph : CityGraph, nbTimes : int, budget : int) -> Attack :
	"""Each time, it removes a random edge from the graph until the budget is reached or no more edges are available"""

	attacks = Attack(graph)
	edges = list(graph.edges())
	
	for time in range(nbTimes) :
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

def sorted_attack(graph : CityGraph, nbTimes : int, budget : int, sort_by : callable) -> Attack :
	"""Each time, it removes the edge that is the most affected by the sort_by function until the budget is reached or no more edges are available"""

	attacks = Attack(graph)
	edges = list(graph.edges())
	for i, edge in enumerate(edges) :
		edges[i] = graph.find_edge_index(edge[0], edge[1])
	edges.sort(key = sort_by, reverse = True)

	for time in range(nbTimes) :
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

def min_lanes_attack(graph : CityGraph, nbTimes : int, budget : int) -> Attack :
	"""Each time, it removes the edge with the least lanes until the budget is reached or no more edges are available"""
	return sorted_attack(graph, nbTimes, budget, lambda x : -graph.get_nb_lanes_of_edge(x))

def max_lanes_attack(graph : CityGraph, nbTimes : int, budget : int) -> Attack :
	"""Each time, it removes the edge with the most lanes until the budget is reached or no more edges are available"""
	return sorted_attack(graph, nbTimes, budget, lambda x : graph.get_nb_lanes_of_edge(x))

def betweenness_centralities_attack(graph : CityGraph, nbTimes : int, budget : int) -> Attack :
	"""Each time, it removes the edge with the highest betweenness centrality until the budget is reached or no more edges are available"""
	return sorted_attack(graph, nbTimes, budget, lambda x : graph.get_edge_betweenness_centrality(x))

def moving_attack(graph : CityGraph, nbTimes : int, budget : int) -> Attack :
	"""We start with random edges, and then with each time the attacks are moved to a neighbor edge"""
	edges = list(graph.edges())
	
	# First time
	attacks = random_attack(graph, 1, budget)
	
	# Next times
	# First, find the neighbours of the concerned edges
	# To find the neighbours, we find the edges connected to either of the nodes of the edge
	neighbours = {}
	for edge_idx in range(graph.nb_edges()) :
		edge = graph.get_edge_at(edge_idx)
		neighbours[edge_idx] = set()
		for node in edge :
			for edge2 in graph.graph.edges(node) :
				neighbours[edge_idx].add(graph.find_edge_index(edge2[0], edge2[1]))		

	# Remove the edge itself from it's neighbours
	for (k, v) in neighbours.items():
		if k in v:
			v.remove(k)

	# Move the people around, but only add the attack if all the lanes are occupied
	nb_people_on_edges = [0 for _ in range(graph.nb_edges())]
	for attack in attacks.attacks :
		nb_people_on_edges[attack[1]] += graph.get_nb_lanes_of_edge(attack[1])

	print(nb_people_on_edges)


	for time in range(1, nbTimes) :
		for edge_idx in range(graph.nb_edges()) :
			if nb_people_on_edges[edge_idx] > 0 :
				# Move the people
				edge_idx2 = random.choice(list(neighbours[edge_idx]))
				nb_people_on_edges[edge_idx] -= graph.get_nb_lanes_of_edge(edge_idx)
				nb_people_on_edges[edge_idx2] += graph.get_nb_lanes_of_edge(edge_idx)
				# Add the attack if all the lanes are occupied, dont move if not enough or already full
				if nb_people_on_edges[edge_idx2] >= graph.get_nb_lanes_of_edge(edge_idx2) :
					attacks.add_attack(time, edge_idx2)
					nb_people_on_edges[edge_idx2] -= graph.get_nb_lanes_of_edge(edge_idx2)
					nb_people_on_edges[edge_idx] += graph.get_nb_lanes_of_edge(edge_idx)
				else :
					attacks.add_attack(time, edge_idx)
				
	# Add the last attack
	for edge_idx in range(graph.nb_edges()) :
		if nb_people_on_edges[edge_idx] > 0 :
			attacks.add_attack(nbTimes, edge_idx)
			

	return attacks


graph = CityGraph(ville)
graph.print_stats()
# graph.show()

# a = random_attack(graph, 50, 50)
# a.write_to_file("test_random_attack.txt")
# a = min_lanes_attack(graph, 10, 10)
# a.write_to_file("test_min_lanes_attack.txt")
# a = max_lanes_attack(graph, 10, 10)
# a.write_to_file("test_max_lanes_attack.txt")
# a = betweenness_centralities_attack(graph, 10, 10)
# a.write_to_file("test_betweenness_centralities_attack.txt")
a = moving_attack(graph, 10, 100)
a.write_to_file("test_moving_attack.txt")