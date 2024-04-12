import osmnx as ox
import sys
from CityGraph import CityGraph

def get_city_edges(city_name):
    # Importer le graphe de la ville
    city_graph = CityGraph(city_name)
    num_edges = city_graph.nb_edges()

    return num_edges

if __name__ == "__main__":
    city_name = sys.argv[1]
    num_edges = get_city_edges(city_name)

    # Communiquer le nombre d'arêtes au script appelant
    print(num_edges)