import osmnx as ox
import networkx as nx

import osmnx as ox
import networkx as nx
import numpy as np
import scipy as sp


LOWEST_MAX_SPEED=30

class CityGraph :
	"""
	Represents a graph of a city, with some useful methods
	--- Attributes ---
	city_name : name of the city
	graph : the graph itself
	nodes_idx_map : a dict that maps a node to its index in the graph
	projected_graph : the graph projected on a plane
	lanes_data : the number of lanes of each edge indexed by the edge index
	"""
	city_name : str
	graph : nx.Graph
	nodes_idx_map : dict[int, int]
	projected_graph : nx.Graph = None
	lanes_data : list[int] = None
	betweenness_centralities : list[float] = None

	def __init__(self, city_name : str, network_type : str = "drive") :
		"""Loads a graph from OSmnx through its name"""
		ox.settings.osm_xml_way_tags=["highway", "lanes"]
		self.city_name = city_name
		self.graph = ox.graph_from_place(city_name, network_type=network_type)
		self.nodes_idx_map = { edge : i  for i, edge in enumerate(self.graph.nodes())}

	def get_node_index(self, node : int) :
		"""Returns the index in the matrix of the node through it's OSmnx numero"""
		return self.nodes_idx_map[node]

	def show(self) :
		"""Opens an image of the graph"""
		ox.plot_graph(self.graph)

	def print_stats(self) :
		"""Prints some stats about the graph"""
		print(ox.basic_stats(self.graph))

	def get_lanes_data(self) :
		"""Returns the number of lanes of each edge"""
		if self.lanes_data is None :
			graph_lanes = self.graph.edges.data("lanes")
			lanes_data = []
			for edge in graph_lanes :
				if edge[2] != None :
					lanes_data.append(int(edge[2]))
				else :
					lanes_data.append(1)
			self.lanes_data = lanes_data

		return self.lanes_data
	
	def get_betweenness_centralities(self) :
		"""Returns the betweenness centralities of each edge"""
		if self.betweenness_centralities is None :
			self.betweenness_centralities = nx.edge_betweenness_centrality(self.get_projected_graph())
		return self.betweenness_centralities

	def get_projected_graph(self) :
		"""Returns the projected graph"""
		if self.projected_graph is None :
			self.projected_graph = ox.project_graph(self.graph)
		return self.projected_graph

	def edges(self) :
		"""Returns the edges of the graph"""
		return self.graph.edges()

	def nb_nodes(self) :
		"""Returns the number of nodes in the graph"""
		return self.graph.size()

	def nb_edges(self) :
		"""Returns the number of edges in the graph"""
		return self.graph.number_of_edges()

	# TODO : fait la meme chose que get_edge_index
	def get_edge_at(self, edge_idx : int) :
		"""Accesses an edge through its index"""
		return list(self.graph.edges())[edge_idx]
	
	def get_nb_lanes_of_edge(self, edge_idx : int) -> int:
		"""Returns the number of lanes of an edge"""
		edges = self.get_lanes_data()
		return edges[edge_idx]
	
	def get_edge_betweenness_centrality(self, edge_idx : int) -> float:
		"""Returns the betweenness centrality of an edge"""
		edges = self.get_betweenness_centralities()
		edge = self.get_edge_at(edge_idx)
		edge = (edge[0], edge[1], 0) # keys are tuples with a third 0 for some reason
		return edges[edge]

	def find_edge_index(self, node0 : int, node1 : int) -> int:
		"""Returns the index of an edge through its nodes"""
		return list(self.graph.edges()).index((node0, node1))
	
	def get_neighbours_of_edge(self, edge_idx : int) -> list[int]:
		"""Returns the neighbours of an edge"""
		print(edge_idx)
		edge = self.get_edge_at(edge_idx)
		return list(self.graph.neighbors(edge[0])) + list(self.graph.neighbors(edge[1]))

	def get_edge_index(self, edge_idx : int) :
		"""Accesses an edge through its index"""
		return list(self.graph.edges())[edge_idx]

	def write_to_file(self, file_path : str, delta=1/3600) -> None : # by default, delta convert the length to have the speed correzsponding to m/s
		nodeList=list(self.graph.nodes); # Should be changed
		csr=nx.to_scipy_sparse_array(self.graph, format='csr'); # format=csr not needed
		V=np.ndarray(len(csr.indices), dtype=np.uint32)
		

		for i in range(len(csr.indptr)-1):
			for j in range(csr.indptr[i], csr.indptr[i+1]):
				weights=[]
				d=[] # in case of no maxspeed
				# print(list(self.graph.succ.keys())[0],nodeList[i])
				print(self.graph.succ[nodeList[i]], nodeList[csr.indices[j]])
				for e in self.graph.succ[nodeList[i]][nodeList[csr.indices[j]]].values():
					if "weight" in e.keys():
						weights.append(e["weight"])
					elif "maxspeed" in e.keys():
						weights.append(int(e["length"]*delta)/cast_tmp(e["maxspeed"])); # TODO Remove this cast
					else:
						d.append(e["length"])
				if(len(weights)==0):
					V[j]=int(np.min(d)*100000)/LOWEST_MAX_SPEED;
				else:
					V[j]=np.min(weights);

		"""
		i=0;
		print(nodeList)
		for j in range(len(csr.indices)):
			weights=[]
			d=[] # in case of no maxspeed
			print(list(self.graph.succ.keys())[0],nodeList[i])
			print(self.graph.succ[nodeList[i]], nodeList[i])
			for e in self.graph.succ[nodeList[i]][nodeList[j]].values():
				if "weight" in e.keys():
					weights.append(e["weight"])
				elif "maxspeed" in e.keys():
					weights.append(int(e["length"]*100000)/e["maxspeed"]);
				else:
					d.append(e["length"])
			if(len(weights)==0):
				V[j]=int(np.min(d)*100000)/e["maxspeed"];
			else:
				V[j]=np.min(weights);
			j+=1;
			while((i+1<len(csr.indptr)) and (j>=csr.indptr[i+1])): # if ? (while in case of a node without succ)
				i+=1;
		"""

		f=open(file_path, "w");
		# f.write("%d %d %d\n".format(len(V), len(csr.indices), len(csr.indptr)));
		f.write(" ".join([str(x) for x in (len(V), len(csr.indices), len(csr.indptr))])+"\n") # TODO change this line
		for i in V:
			f.write(str(i)+" ");
		f.write("\n");

		for i in csr.indices:
			f.write(str(i)+" ");
		f.write("\n");

		for i in csr.indptr:
			f.write(str(i)+" ");
		f.write("\n");

		f.close();

def cast_tmp(var):
	"""TODO : Change this function"""
	if(type(var)==int):
		return var
	elif((type(var)==float) or (type(var)==str)):
		return int(var)
	elif((type(var)==list) or (type(var)==tuple)):
		return np.min([cast_tmp(x) for x in var])

# Exemple utilisation
# graph = CityGraph("Sainte")
# graph.write_to_file("../../result.txt");

# Exemple utilisation
# graph = CityGraph("Paris")
# graph.print_stats()
# graph.show()
