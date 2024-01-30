import osmnx as ox

class CityGraph :
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

	def read_from_file(file_path : str) -> CityGraph :
		# TODO
		return None

	def write_to_file(self, file_path : str) -> None :
		# TODO
		pass

# Exemple utilisation
# graph = CityGraph("Paris")
# graph.print_stats()
# graph.show()
