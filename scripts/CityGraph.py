import osmnx as ox
import networkx as nx

class CityGraph :
	"""
	Represents a graph of a city, with some useful methods
	--- Attributes ---
	city : name of the city
	graph : the graph itself
	nodes : a dict that maps a node to its index in the graph
	projected_graph : the graph projected on a plane
	"""
	city : str
	graph : nx.Graph
	nodes : dict[int, int]
	projected_graph : nx.Graph = None

	def __init__(self, city : str, network_type : str = "drive") :
		"""Loads a graph from OSmnx through its name"""
		self.city = city
		self.graph = ox.graph_from_place(city, network_type=network_type)
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
