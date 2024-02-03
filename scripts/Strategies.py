import osmnx as ox
import networkx as nx
import random

from CityGraph import CityGraph
from Attack import Attack, print_suppression_edge

# Juste pour test pour l'instant
ville="Saints"

def random_attack(graph : CityGraph, nbTemps : int, budget : int) -> Attack :
	"""A chaque temps, on supprime budget liens choisis aleatoirement"""

	attacks = Attack(graph)
	edges = list(graph.edges())
	
	for i in range(nbTemps) :
		edges_t = edges.copy()
		budget_used : int = 0
		nb_retires : int = 0
		while budget_used < budget :
			edge_idx = random.randint(0, graph.nb_edges() - 1 - nb_retires)
			# print_suppression_edge(graph, edge_idx, i)
			attacks.add_attack(i, edge_idx)
			edges_t.pop(edge_idx)
			nb_retires += 1
			budget_used += 1 + graph.get_nb_lanes_of_edge(edge_idx)

	return attacks
			

graph = CityGraph(ville)
graph.print_stats()
graph.show()

a = random_attack(graph, 50, 50)
a.write_to_file("test_attack.txt")