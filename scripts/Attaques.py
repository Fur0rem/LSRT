import osmnx as ox
import networkx as nx
import random

from CityGraph import CityGraph

# Juste pour test pour l'instant
ville="Saints"

def format_suppression_edge(graph : CityGraph, edge_idx : int, time : int) -> str :
	edge = graph.get_edge_index(edge_idx)
	noeud0 = graph.get_node_index(edge[0])
	noeud1 = graph.get_node_index(edge[1])
	return f"T={time}, suppression de l'arete {edge_idx} entre les noeuds {noeud0} et {noeud1}"

def random_attack(graph : CityGraph, nbTemps : int, budget : int, output_file : str) -> None :
	"""A chaque temps, on supprime budget liens choisis aleatoirement"""
	
	assert budget <= graph.size()

	# TODO : ecriture dans le fichier
	edges = list(graph.edges())
	for i in range(nbTemps) :
		for j in range(budget) :
			edge_idx = random.randint(0, len(edges)-1)
			print(format_suppression_edge(graph, edge_idx, i))
			

graph = CityGraph(ville)
graph.print_stats()
graph.show()

random_attack(graph, 10, 5, "test.txt")