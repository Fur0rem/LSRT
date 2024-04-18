import osmnx as ox
import sys
from CityGraph import CityGraph

if __name__ == "__main__":
    city_name = sys.argv[1]
    city_graph = CityGraph(city_name)
    num_edges = city_graph.graph.number_of_edges()
    num_nodes = city_graph.graph.number_of_nodes()
    diameter = city_graph.diameter()

    # Communiquer le nombre d'arêtes au script appelant avec les print
    print(diameter)
    print(num_nodes)
    print(num_edges)