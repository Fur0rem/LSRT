from CityGraph import CityGraph

# Juste pour test pour l'instant
ville="Saints"

# TODO : rename shit better
class StaticAttack :
    graph : CityGraph
    attacked_links : list[(int, int)]
    delta : int

    def __init__(self, graph : CityGraph, delta : int = 0) :
        self.graph = graph
        self.attacked_links = []
        self.delta = delta

    def set_delta(self, delta : int) -> None :
        """Sets the delta"""
        self.delta = delta

    def block_link(self, node_from : int, node_to : int) -> None :
        """Blocks a link"""
        self.attacked_links.append((node_from, node_to))

    def write_to_file(self, file_path : str) -> None :
        """Writes the attacks to a file"""
        with open(file_path, "w") as file :
            file.write("S\n")
            file.write(f"{len(self.attacked_links)} {self.delta}\n")

            deleted_neighbours = [[] for i in range(self.graph.nb_nodes())]
            for block in self.attacked_links :
                print(block)
                deleted_neighbours[block[0]].append(block[1])
            
            for i in range(self.graph.nb_nodes()):
                file.write(f"{' '.join(map(str,deleted_neighbours[i]))}\n")

        print(f"Attack has been written to {file_path}")



class Attack :
    """
    Represents an attack on a graph
    --- Attributes ---
    graph : the graph to attack (READ-ONLY)
    attacks : the attacks to perform
    """

    graph : CityGraph
    attacks : list[tuple[int, int]] # Format : [time, edge_idx]
    delta : int

    def __init__(self, graph : CityGraph, delta : int = 0) :
        self.graph = graph
        self.attacks = []
        self.delta = delta

    def set_delta(self, delta : int) -> None :
        """Sets the delta"""
        self.delta = delta
    
    def add_attack(self, time : int, edge_idx : int) -> None :
        """Adds an attack to the list"""
        self.attacks.append((time, edge_idx))

    def write_to_file(self, file_path : str) -> None :
        """Writes the attacks to a file"""

        # Map the edges to their indexes
        edges_of_graph = [(self.graph.get_node_index(edge[0]), self.graph.get_node_index(edge[1])) for edge in self.graph.edges()]

        # Conversion
        edges_deleted : dict[int, set[int]] = { edge : set() for edge in edges_of_graph}
        for time, edge_idx in self.attacks :
            node0 = self.graph.get_node_index(self.graph.get_edge_at(edge_idx)[0])
            node1 = self.graph.get_node_index(self.graph.get_edge_at(edge_idx)[1])
            edges_deleted[(node0, node1)].add(time)

        # Sorting for easier parsing in the future    
        for edge in edges_deleted :
            edges_deleted[edge] = list(edges_deleted[edge])
            edges_deleted[edge].sort()

        edges_deleted : list[tuple[tuple[int, int], list[int]]] = [((edge[0], edge[1]), edges_deleted[edge]) for edge in edges_deleted]
        edges_deleted.sort(key = lambda x : (x[0][0], x[0][1]))
        #print(edges_deleted)
        
        # Writing
        with open(file_path, "w") as file :
            max_time = self.attacks[-1][0] + 1
            file.write("D\n")
            file.write(f"{len(edges_deleted)} {self.delta} {max_time}\n")
            for edge, times in edges_deleted :
                file.write(f"{len(times)} {' '.join(map(str, times))}\n")
            
        print(f"Attacks have been written to {file_path}")
                
def print_suppression_edge(graph : CityGraph, edge_idx : int, time : int):
    edge = graph.get_edge_at(edge_idx)
    noeud0 = graph.get_node_index(edge[0])
    noeud1 = graph.get_node_index(edge[1])
    print(f"T={time}, suppression de l'arete {edge_idx} entre les noeuds {noeud0} et {noeud1}")
