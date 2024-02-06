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
	"""
	city_name : str
	graph : nx.Graph
	nodes_idx_map : dict[int, int]
	projected_graph : nx.Graph = None

	def __init__(self, City : str, network_type : str = "drive") :
		"""Loads a graph from OSmnx through its name"""
		self.City = City
		self.graph = ox.graph_from_place(City, network_type=network_type)
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

	def get_edge_index(self, edge_idx : int) :
		"""Accesses an edge through its index"""
		return list(self.graph.edges())[edge_idx]

	@staticmethod
	def read_from_file(file_path : str) :
		# TODO
		return None

	def write_to_file(self, file_path : str) -> None :
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
						weights.append(int(e["length"]*100000)/cast_tmp(e["maxspeed"])); # TODO Remove this cast
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
graph = CityGraph("Sainte")
graph.write_to_file("../../result.txt");