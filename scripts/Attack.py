import osmnx as ox
import networkx as nx
import random

from CityGraph import CityGraph

# Juste pour test pour l'instant
ville="Saints"

class Attack :
    """
    Represents an attack on a graph
    --- Attributes ---
    graph : the graph to attack (READ-ONLY)
    attacks : the attacks to perform
    """

    graph : CityGraph
    attacks : list[int] # Format : [time, edge_idx]
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
        
        #print(self.graph.nodes_idx_map)
        #edges_of_graph = list(self.graph.graph.edges())
        #print(edges_of_graph)
        # Map the edges to their indexes
        edges_of_graph = [(self.graph.get_node_index(edge[0]), self.graph.get_node_index(edge[1])) for edge in self.graph.edges()]
        #edges_of_graph.sort(key = lambda x : (x[0], x[1]))

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
            file.write(f"{len(edges_deleted)} {self.delta}\n")
            for edge, times in edges_deleted :
                file.write(f"{len(times)} {' '.join(map(str, times))}\n")
        
        print(f"Attacks have been written to {file_path}")
                
def print_suppression_edge(graph : CityGraph, edge_idx : int, time : int):
    edge = graph.get_edge_at(edge_idx)
    noeud0 = graph.get_node_index(edge[0])
    noeud1 = graph.get_node_index(edge[1])
    print(f"T={time}, suppression de l'arete {edge_idx} entre les noeuds {noeud0} et {noeud1}")
