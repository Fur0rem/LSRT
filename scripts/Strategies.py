import random
import networkx as nx

from CityGraph import CityGraph
from Attack import Attack, print_suppression_edge


# Juste pour test pour l'instant
ville="Melun"

def random_attack(graph : CityGraph, nbTimes : int, budget : int, costPerLane : bool) -> Attack :
    """Each time, it removes a random edge from the graph until the budget is reached or no more edges are available"""

    costFunc = graph.get_nb_lanes_of_edge if costPerLane else lambda x : 1

    attacks = Attack(graph)
    edges = list(graph.edges())
    
    for time in range(nbTimes) :
        edges_t = edges.copy()
        budget_used = 0
        nb_retires = 0
        while budget_used < budget and len(edges_t) > 0 :
            edge_idx = random.randint(0, graph.nb_edges() - (nb_retires + 1))
            cost = costFunc(edge_idx)
            if cost <= budget - budget_used :
                attacks.add_attack(time, edge_idx)
                budget_used += cost
                nb_retires += 1
            edges_t.pop(edge_idx)

    return attacks

def sorted_attack(graph : CityGraph, nbTimes : int, budget : int, sort_by : callable, costPerLane : bool) -> Attack :
    """Each time, it removes the edge that is the most affected by the sort_by function until the budget is reached or no more edges are available"""

    costFunc = graph.get_nb_lanes_of_edge if costPerLane else lambda x : 1

    attacks = Attack(graph)
    edges = list(graph.edges())
    for i, edge in enumerate(edges) :
        edges[i] = graph.find_edge_index(edge[0], edge[1])
    edges.sort(key = sort_by, reverse = True)

    for time in range(nbTimes) :
        edges_t = edges.copy()
        budget_used = 0
        nb_retires = 0
        while budget_used < budget and len(edges_t) > 0 :
            edge_idx = edges_t.pop(0)
            cost = costFunc(edge_idx)
            if cost <= budget - budget_used :
                attacks.add_attack(time, edge_idx)
                budget_used += cost
                nb_retires += 1
            edges_t.pop(edge_idx)

    return attacks

def min_lanes_attack(graph : CityGraph, nbTimes : int, budget : int, costPerLane : bool) -> Attack :
    """Each time, it removes the edge with the least lanes until the budget is reached or no more edges are available"""
    return sorted_attack(graph, nbTimes, budget, lambda x : -graph.get_nb_lanes_of_edge(x), costPerLane)

def max_lanes_attack(graph : CityGraph, nbTimes : int, budget : int, costPerLane : bool) -> Attack :
    """Each time, it removes the edge with the most lanes until the budget is reached or no more edges are available"""
    return sorted_attack(graph, nbTimes, budget, lambda x : graph.get_nb_lanes_of_edge(x), costPerLane)

def betweenness_centralities_attack(graph : CityGraph, nbTimes : int, budget : int, costPerLane : bool) -> Attack :
    """Each time, it removes the edge with the highest betweenness centrality until the budget is reached or no more edges are available"""
    return sorted_attack(graph, nbTimes, budget, lambda x : graph.get_edge_betweenness_centrality(x), costPerLane)

def moving_attack(graph : CityGraph, nbTimes : int, budget : int, costPerLane : bool) -> Attack :
    """We start with random edges, and then with each time the attacks are moved to a neighbor edge"""
    
    costFunc = graph.get_nb_lanes_of_edge if costPerLane else lambda x : 1
    edges = list(graph.edges())
    
    # First time
    attacks = random_attack(graph, 1, budget, costPerLane)
    
    # Next times
    # First, find the neighbours of the concerned edges
    # To find the neighbours, we find the edges connected to either of the nodes of the edge
    neighbours = {}
    for edge_idx in range(graph.nb_edges()) :
        edge = graph.get_edge_at(edge_idx)
        neighbours[edge_idx] = set()
        for node in edge :
            for edge2 in graph.graph.edges(node) :
                neighbours[edge_idx].add(graph.find_edge_index(edge2[0], edge2[1]))        

    # Remove the edge itself from it's neighbours
    for (k, v) in neighbours.items():
        if k in v:
            v.remove(k)

    # Move the people around, but only add the attack if all the lanes are occupied
    nb_people_on_edges = [0 for _ in range(graph.nb_edges())]
    for attack in attacks.attacks :
        nb_people_on_edges[attack[1]] += costFunc(attack[1])

    #print(nb_people_on_edges)


    for time in range(1, nbTimes) :
        for edge_idx in range(graph.nb_edges()) :
            if nb_people_on_edges[edge_idx] > 0 :
                # Move the people
                
                # If there are no neighbours, stay on the edge
                if len(neighbours[edge_idx]) == 0 :
                    attacks.add_attack(time, edge_idx)
                    continue

                edge_idx2 = random.choice(list(neighbours[edge_idx]))
                nb_people_on_edges[edge_idx] -= costFunc(edge_idx)
                nb_people_on_edges[edge_idx2] += costFunc(edge_idx)
                # Add the attack if all the lanes are occupied, dont move if not enough or already full
                if nb_people_on_edges[edge_idx2] >= costFunc(edge_idx2) :
                    attacks.add_attack(time, edge_idx2)
                    nb_people_on_edges[edge_idx2] -= costFunc(edge_idx2)
                    nb_people_on_edges[edge_idx] += costFunc(edge_idx)
                else :
                    attacks.add_attack(time, edge_idx)
                
    # Add the last attack
    for edge_idx in range(graph.nb_edges()) :
        if nb_people_on_edges[edge_idx] > 0 :
            attacks.add_attack(nbTimes, edge_idx)
            

    return attacks


def connex_attack(graph : CityGraph, nbTimes : int, budget : int) -> Attack :
    """We remove edges which cut the graph into two parts recursively
    Algorithm reference : https://stackoverflow.com/questions/11218746/bridges-in-a-connected-graph"""
    

    def is_bridge(graph : CityGraph, node0 : int, node1 : int) -> bool :
        """Returns True if the edge is a bridge"""
        # TODO : ca marche pas pour les graphes orientes car networkx veut me rendre malheureux    
        # Remove the edge
        graph.graph.remove_edge(node0, node1)
        # Check if the graph is still connected
        is_connected = nx.is_connected(graph.graph)
        # Add the edge back
        graph.graph.add_edge(node0, node1)
        return not is_connected

    attacks = Attack(graph)
    edges = list(graph.edges())

    # Find the bridge with the highest betweenness centrality
    # Then remove it
    for time in range(nbTimes) :
        # Find the bridge with the highest betweenness centrality
        bridge = None
        max_betweenness = 0
        for edge_idx in range(graph.nb_edges()) :
            edge = graph.get_edge_at(edge_idx)
            if is_bridge(graph, edge[0], edge[1]) :
                betweenness = graph.get_edge_betweenness_centrality(edge_idx)
                if betweenness > max_betweenness :
                    bridge = edge_idx
                    max_betweenness = betweenness
        if bridge is not None :
            attacks.add_attack(time, bridge)
        else :
            break
        
        print_suppression_edge(graph, bridge)

    return attacks

#graph = CityGraph(ville)
#graph.print_stats()
# graph.show() # je sais pas comment fermer une fenetre sous sway au secours

#a = random_attack(graph, 10, 10, True)
#a.write_to_file("test_random_attack.txt")
# a = min_lanes_attack(graph, 10, 10)
# a.write_to_file("test_min_lanes_attack.txt")
# a = max_lanes_attack(graph, 10, 10)
# a.write_to_file("test_max_lanes_attack.txt")
# a = betweenness_centralities_attack(graph, 10, 10)
# a.write_to_file("test_betweenness_centralities_attack.txt")
# a = moving_attack(graph, 10, 100)
# a.write_to_file("test_moving_attack.txt")
# a = connex_attack(graph, 10, 10)
# a.write_to_file("test_connex_attack.txt")

import argparse

def main():
    # nom_ville : str, type_attack : str, nb_times : int, delta : int (default 8), output_file : Optional[str]
    parser = argparse.ArgumentParser(description="Generate an attack on a city graph")
    parser.add_argument("nom_ville", type=str, help="The name of the city")
    parser.add_argument("type_attack", type=str, help="The type of attack, can be 'random', 'min_lanes', 'max_lanes', 'betweenness_centralities', 'moving' or 'connex'")
    parser.add_argument("nb_times", type=int, help="The number of times the attack is repeated")
    parser.add_argument("budget", type=int, help="The budget of the attack", nargs="?", default=100)
    parser.add_argument("delta", type=int, help="The variation frequency of the attack", nargs="?", default=8)
    parser.add_argument("output_file", type=str, help="The output file", nargs="?", default = None)

    args = parser.parse_args()
    args.output_file = args.output_file if args.output_file is not None else f"{args.nom_ville}_{args.type_attack}_{args.nb_times}_{args.budget}_{args.delta}.txt"
    graph = CityGraph(args.nom_ville)

    # Peut etre transformer en variable dynamiquement si ca devient trop
    switch = {
        "random" : random_attack,
        "min_lanes" : min_lanes_attack,
        "max_lanes" : max_lanes_attack,
        "betweenness_centralities" : betweenness_centralities_attack,
        "moving" : moving_attack,
        "connex" : connex_attack
    }

    if args.type_attack not in switch :
        print(f"Error : {args.type_attack} is not a valid attack type")
        return
    
    # Write graph to file
    graph.write_to_file(f"tests_files/{args.nom_ville}.txt")
    
    # Write attack to file
    a = switch[args.type_attack](graph, args.nb_times, args.budget, True)
    a.set_delta(args.delta)
    a.write_to_file(f"tests_files/{args.output_file}.txt")


if __name__ == "__main__" :
    main()