import osmnx as ox
import networkx as nx
import numpy as np
import re

LOWEST_MAX_SPEED=30
STEP=60 # The duration of a weigth of 1 in the sparse matrix (in seconds)
        # 1 minute here

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
	list_edges : list[int]

	def __init__(self, city_name : str, network_type : str = "drive") :
		"""Loads a graph from OSmnx through its name"""
		print("Loading graph of", city_name)
		ox.settings.osm_xml_way_tags=["highway", "lanes"]
		self.city_name = city_name
		self.graph = ox.graph_from_place(city_name, network_type=network_type)
		print("Graph loaded")
		self.nodes_idx_map = { edge : i  for i, edge in enumerate(self.graph.nodes())}

		print("Map created")
		for nodes,edge in self.graph.edges.items():
			if("length" in edge.keys()):
				length=edge["length"]
			else:
				length=euclidian_distance(nodes[0], nodes[1])

			if "maxspeed" in edge.keys() :
				speed=normalize_speed(edge["maxspeed"])
			else:
				speed=LOWEST_MAX_SPEED
 
			# TODO : put the coef in normalize_speed
			edge["weight"]=max(int((length*3.6)/(speed)), 1)

		print("Normalised the data")
		self.list_edges = list(self.graph.edges())
		print("Graph fully done")
		print("Graph has", self.nb_nodes(), "nodes and", self.nb_edges(), "edges")

	def get_node_index(self, node : int) :
		"""Returns the index in the matrix of the node through it's OSmnx numero"""
		return self.nodes_idx_map[node]
	
	# def diameter(self):
	# 	"""The diameter of the graph"""
	# 	# Convert the graph to be non oriented else networkx is mad >:(
	# 	tmp_graph = None
	# 	if self.graph.is_directed() :
	# 		tmp_graph = self.graph.to_undirected()
	# 	else :
	# 		tmp_graph = self.graph

	# 	# If the graph is not connected, choose the biggest connected component
	# 	max_diameter = 0
	# 	for component in nx.connected_components(tmp_graph) :
	# 		if nx.diameter(tmp_graph.subgraph(component)) > max_diameter :
	# 			max_diameter = nx.diameter(tmp_graph.subgraph(component))
	# 	return max_diameter

	def show(self) :
		"""Opens an image of the graph"""
		ox.plot_graph(self.graph)

	def print_stats(self) :
		"""Prints some stats about the graph"""
		print(ox.basic_stats(self.graph))

	def get_lanes_data(self) :
		"""Returns the number of lanes of each edge"""
		if self.lanes_data is None :
			graph_lanes = self.graph.edges.data("lanes", default = 1)
			lanes_data = []
			for edge in graph_lanes :
				#print(edge)

				# Sometimes it's a list for some reason
				if type(edge[2]) == list :
					sum_list = sum([int(x) for x in edge[2]])
					lanes_data.append(int(sum_list/len(edge[2])))
				else :
					lanes_data.append(int(edge[2]))
			self.lanes_data = lanes_data

		#print(self.lanes_data)
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
		return self.graph.number_of_nodes()

	def nb_edges(self) :
		"""Returns the number of edges in the graph"""
		return self.graph.number_of_edges()

	def diameter(self):
		"""Not finished"""
		#return nx.diameter(self.graph, weight="weight")
		# Make the graph non oriented
		tmp_graph = None
		if self.graph.is_directed() :
			tmp_graph = self.graph.to_undirected()
		else :
			tmp_graph = self.graph

		# If the graph is not connected, choose the biggest connected component
		max_diameter = 0
		for component in nx.connected_components(tmp_graph) :
			if nx.diameter(tmp_graph.subgraph(component)) > max_diameter :
				max_diameter = nx.diameter(tmp_graph.subgraph(component), weight="weight")

		return max_diameter
	

	# TODO : fait la meme chose que get_edge_index
	def get_edge_at(self, edge_idx : int) :
		"""Accesses an edge through its index"""
		return self.list_edges[edge_idx]
	
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

	def write_to_file(self, file_path : str) -> None :
		nodeList=list(self.graph.nodes); # Should be changed
		csr=nx.to_scipy_sparse_array(self.graph, format='csr'); # format=csr not needed
		V=np.ndarray(len(csr.indices), dtype=np.uint32)
		coef=STEP/3.6 # TODO : put the coef in normalize_speed

		# For each neighbors
		for i in range(len(csr.indptr)-1):
			for j in range(csr.indptr[i], csr.indptr[i+1]):
				weights=[]
				for edge in self.graph.succ[nodeList[i]][nodeList[csr.indices[j]]].values():
					weights.append(edge["weight"])

				V[j]=np.min(weights)

		f=open(file_path, "w");
		f.write(" ".join([str(x) for x in (len(V), len(csr.indptr))])+"\n")
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

def normalize_speed(var):
	"""Make sure that the weigth of a node become an int with the same unit, whatever it was"""
	if(type(var)==int):
		return var
	elif((type(var)==float) or ((type(var)==str)) and var.isnumeric()):
		return int(var)
	elif((type(var)==list) or (type(var)==tuple)):
		return np.min([normalize_speed(x) for x in var if(x!='walk')])
	elif(type(var)==str):
		# Try to parse it as a speed
		zone=re.search('zone:?([0-9]*)', var)
		mph=re.search('([0-9]*) mph', var)
		de=re.search('[A-Z]{2}:([0-9]*)', var)
		if(zone!=None):
			return int(zone[1])
		elif(mph!=None):
			return int(mph[1])*1.609344
		elif(de!=None):
			return int(de[1])	
	# print("Error : ", type(var), (var))
	return LOWEST_MAX_SPEED

def euclidian_distance(a,b):
	"""Give the euclidian distance between two nodes"""
	return np.sqrt((a["x"]-b["x"])**2+(a["y"]-b["y"])**2)

# Exemple utilisation
# graph = CityGraph("Sainte")
# graph.write_to_file("../../result.txt");

# Exemple utilisation
# graph = CityGraph("Paris")
# graph.print_stats()
# graph.show()
