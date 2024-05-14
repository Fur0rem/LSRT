import matplotlib.pyplot as plt
import numpy as np
import osmnx as ox
from CityGraph import CityGraph

ville = "Melun"

# Load the graph

# change text size
plt.rc('font', size=18)

# change the size of the plot
plt.figure(figsize=(10, 6))

ox.settings.osm_xml_way_tags=["highway", "lanes", "degree"]
graph = ox.graph_from_place(ville, network_type="drive")
degrees = [graph.degree(node) for node in graph.nodes()]
plt.hist(degrees, bins=range(0, max(degrees) + 1, 1), edgecolor='black')
plt.xlabel('Degré')
plt.ylabel('Nombre de noeuds')
#plt.title(f'Histogramme des degrés des noeuds de {ville}')
plt.savefig(f'histogramme_degres_{ville}.pdf')
plt.show()

city_graph = CityGraph(ville)
# change the size of the plot
plt.figure(figsize=(10, 6))
lengths = [city_graph.graph.edges[edge]["length"] for edge in city_graph.graph.edges]
plt.hist(lengths, bins=range(0, int(max(lengths)) + 1, 1), edgecolor='black')
plt.xlabel('Temps de traversée (pas de temps)')
plt.ylabel('Nombre de liens')
#plt.title(f'Histogramme des longueurs des liens de {ville}')
plt.savefig(f'histogramme_longueurs_{ville}.pdf')
plt.show()

print("Degré moyen : ", np.mean(degrees))
print("Temps de traversée moyen : ", np.mean(lengths))
print("Nombre de liens : ", len(lengths))
print("Nombre de noeuds : ", len(degrees))
