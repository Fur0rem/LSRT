import osmnx as ox
import networkx as nx
import numpy as np
import scipy as sp
import sys
import random

# Juste pour test pour l'instant
ville="Saints"

class NetworkGraph :
	ville : str
	graph : nx.Graph
	nodes : dict[int, int]
	projected_graph : nx.Graph = None

	def __init__(self, ville : str, network_type : str = "drive") :
		self.ville = ville
		self.graph = ox.graph_from_place(ville, network_type=network_type)
		self.nodes = { edge : i  for i, edge in enumerate(self.graph.nodes())}

	def get_node_index(self, node : int) :
		return self.nodes[node]

	def get_node(self, index : int) :
		return self.nodes[index]

	def show(self) :
		ox.plot_graph(self.graph)

	def print_stats(self) :
		print(ox.basic_stats(self.graph))

	def get_projected_graph(self) :
		if self.projected_graph is None :
			self.projected_graph = ox.project_graph(self.graph)
		return self.projected_graph

	def edges(self) :
		return self.graph.edges()

	def size(self) :
		return self.graph.size()

	def get_edge_index(self, edge_idx : int) :
		return list(self.graph.edges())[edge_idx]
	

def format_suppression_edge(graph : NetworkGraph, edge_idx : int, time : int) -> str :
	edge = graph.get_edge_index(edge_idx)
	noeud0 = graph.get_node_index(edge[0])
	noeud1 = graph.get_node_index(edge[1])
	return f"T={time}, suppression de l'arete {edge_idx} entre les noeuds {noeud0} et {noeud1}"

def random_attack(graph : NetworkGraph, nbTemps : int, budget : int, output_file : str) -> None :
	"""A chaque temps, on supprime budget liens choisis aleatoirement"""
	
	assert budget <= graph.size()

	# TODO : ecriture dans le fichier
	edges = list(graph.edges())
	for i in range(nbTemps) :
		for j in range(budget) :
			edge_idx = random.randint(0, len(edges)-1)
			print(format_suppression_edge(graph, edge_idx, i))
			

graph = NetworkGraph(ville)
graph.print_stats()
graph.show()

random_attack(graph, 10, 5, "test.txt")