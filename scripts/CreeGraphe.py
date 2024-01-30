import osmnx
import numpy as np
import scipy as sp
import sys

if(__name__=="__main__"):
	if(len(sys.argv)<=2): # 2 pour le fichier out
		sys.stderr.write("Erreur : pas d'arguments")
		exit(1)
	else:
		ville=sys.argv[1]
else:
	ville=input("Entrez un nom de ville : ")

graph=osmnx.graph_from_place(ville);
nbEdges=graph.size();
nbNodes=len(graph.nodes)
colIndex=np.ndarray(nbEdges);
rowIndex=np.ndarray(nbNodes)

nodeToIndice={} # label -> indice dans matrice
i=0;
for u in graph.nodes:
	nodeToIndice[u]=i;
	print(len(nodeToIndice))
	i+=1

# Remplissage des tableaux
# V ? (maybe le nb de rues parallèles car non ponderes dans notre cas)
i=0;
j=0;
for u in graph.nodes:
	rowIndex[nodeToIndice[u]]=i;
	for v in graph.succ[u]: # Ne prends ps en compte les rues paralleles
		colIndex[nodeToIndice[v]]=j;
		# if not v in nodeToIndice.keys():
			# print(v);
		print(v,rowIndex[nodeToIndice[v]])
		j+=1;
	i+=1;

# TODO write