import osmnx as ox
import networkx as nx

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
		# TODO
		pass

# Exemple utilisation
# graph = CityGraph("Paris")
# graph.print_stats()
# graph.show()
