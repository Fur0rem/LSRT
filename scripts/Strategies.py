import copy
import math
import random
import networkx as nx

from CityGraph import CityGraph
from Attack import DynamicAttack, print_suppression_edge, StaticAttack


# Juste pour test pour l'instant
ville="Melun"

def random_attack(graph : CityGraph, nbTimes : int, budget : int, delta : int, costPerLane : bool) -> DynamicAttack :
    """Each time, it removes a random edge from the graph until the budget is reached or no more edges are available"""

    costFunc = graph.get_nb_lanes_of_edge if costPerLane else lambda x : 1

    attacks = DynamicAttack(graph, nbTimes)
    edges = list(graph.edges())
    
    for time in range(1 + (nbTimes // delta)) :
        edges_copy = edges.copy()
        random.shuffle(edges_copy)
        budget_used = 0
        nb_retires = 0
        while budget_used < budget and len(edges_copy) > 0 :
            edge = edges_copy.pop(0)
            edge_idx = graph.find_edge_index(edge[0], edge[1])
            cost = costFunc(edge_idx)
            if cost <= budget - budget_used :
                attacks.add_attack(time, edge_idx)
                budget_used += cost
                nb_retires += 1

    return attacks

def sorted_attack(graph : CityGraph, nbTimes : int, budget : int, delta : int, sort_by : callable, costPerLane : bool) -> StaticAttack :
    """Each time, it removes the edge that is the most affected by the sort_by function until the budget is reached or no more edges are available"""

    costFunc = graph.get_nb_lanes_of_edge if costPerLane else lambda x : 1

    attack = StaticAttack(graph, nbTimes)
    edges = list(graph.edges())

    for i, edge in enumerate(edges) :
        edges[i] = (graph.find_edge_index(edge[0], edge[1]), graph.get_node_index(edge[0]), graph.get_node_index(edge[1]))
    edges.sort(key = lambda x: sort_by(x[0]), reverse = True)

    budget_used = 0
    nb_retires = 0
    while budget_used < budget and len(edges) > 0 :
        (edge_idx, node_from, node_to) = edges.pop(0)
        cost = costFunc(edge_idx)
        if cost <= budget - budget_used :
            attack.block_link(node_from, node_to)
            budget_used += cost
            nb_retires += 1

    return attack

def min_lanes_attack(graph : CityGraph, nbTimes : int, budget : int, delta : int, costPerLane : bool) -> StaticAttack :
    """Each time, it removes the edge with the least lanes until the budget is reached or no more edges are available"""
    return sorted_attack(graph, nbTimes, budget, delta, lambda x : -graph.get_nb_lanes_of_edge(x), costPerLane)

def max_lanes_attack(graph : CityGraph, nbTimes : int, budget : int, delta : int, costPerLane : bool) -> StaticAttack :
    """Each time, it removes the edge with the most lanes until the budget is reached or no more edges are available"""
    return sorted_attack(graph, nbTimes, budget, delta, lambda x : graph.get_nb_lanes_of_edge(x), costPerLane)

def betweenness_centralities_attack(graph : CityGraph, nbTimes : int, budget : int, delta : int, costPerLane : bool) -> StaticAttack :
    """Each time, it removes the edge with the highest betweenness centrality until the budget is reached or no more edges are available"""
    return sorted_attack(graph, nbTimes, budget, delta, lambda x : graph.get_edge_betweenness_centrality(x), costPerLane)

def longitude_attack(graph : CityGraph, nbTimes : int, budget : int, delta : int, costPerLane : bool) -> StaticAttack :
    """Each time, it removes the edge to the most left (by longitude) until the budget is reached or no more edges are available"""
    def get_edge_longitude(edge_idx : int) -> float :
        edge = graph.get_edge_at(edge_idx)
        return min(graph.graph.nodes[edge[0]]["x"], graph.graph.nodes[edge[1]]["x"])
    
    return sorted_attack(graph, nbTimes, budget, delta, get_edge_longitude, costPerLane)

def identifier_attack(graph : CityGraph, nbTimes : int, budget : int, delta : int, costPerLane : bool) -> StaticAttack :
    """Each time, it removes the edge with the highest index until the budget is reached or no more edges are available"""
    return sorted_attack(graph, nbTimes, budget, delta, lambda x : -x, costPerLane)
    


def random_moving_attack(graph : CityGraph, nbTimes : int, budget : int, delta : int, costPerLane : bool) -> DynamicAttack :
    """We start with random edges, and then with each time the attacks are moved to a neighbor edge"""
    
    costFunc = graph.get_nb_lanes_of_edge if costPerLane else lambda x : 1
    edges = list(graph.edges())
    
    # First time
    attacks = random_attack(graph, 1, budget, delta, costPerLane)
    attacks.set_max_time(nbTimes)
    
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


    for time in range(1, 1 + (nbTimes // delta)) :
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
    # for edge_idx in range(graph.nb_edges()) :
    #     if nb_people_on_edges[edge_idx] > 0 :
    #         attacks.add_attack(nbTimes, edge_idx)            

    return attacks


def no_attack(graph : CityGraph, nbTimes : int, budget : int, delta : int, costPerLane : bool) -> StaticAttack :
    """No attack"""
    return StaticAttack(graph, nbTimes)


# def betweenness_centralities_moving_attack(graph : CityGraph, nbTimes : int, budget : int, delta : int, costPerLane : bool) -> DynamicAttack :
#     """
#     We remove the edges with the highest betweenness centrality.
#     Then we recompute the betweenness centralities for the edges of the new graph with the removed edges.
#     """

#     def remove_edge(graph : CityGraph, edge : tuple) :
#         """Removes an edge from the graph"""
#         graph.graph.remove_edge(edge[0], edge[1])

#     costFunc = graph.get_nb_lanes_of_edge if costPerLane else lambda x : 1

#     attacks = DynamicAttack(graph, nbTimes)
#     betweenness_centralities = nx.edge_betweenness_centrality(graph.graph)
#     graph_backup = copy.deepcopy(graph.graph)
#     edges_deleted_last_time = []

#     for time in range(1 + (nbTimes // delta)) :
#         # Sort the edges by betweenness centrality
#         betweenness_centralities = list(betweenness_centralities.items())
#         betweenness_centralities.sort(key = lambda x : x[1], reverse = True)
#         # Remove all the edges with the highest betweenness centrality with the budget
#         budget_used = 0
#         # Betweenness centralities is smaller than nb_edges, so we can use it as a limit
#         edges_in_case = copy.deepcopy(edges_deleted_last_time)
#         edges_deleted_last_time = set()
#         while budget_used < budget and budget_used < len(betweenness_centralities) :
#             edge = betweenness_centralities[budget_used]
#             attacks.add_attack(time, graph.find_edge_index(edge[0][0], edge[0][1]))
#             remove_edge(graph, edge[0])
#             edges_deleted_last_time.add(edge[0])
#             budget_used += 1
#         while budget_used < budget :
#             edge = edges_in_case.pop()
#             if budget_used + costFunc(edge) <= budget:
#                 attacks.add_attack(time, graph.find_edge_index(edge[0], edge[1]))
#                 remove_edge(graph, edge)
#                 edges_deleted_last_time.add(edge)
#                 budget_used += costFunc(edge)
#         # Recompute the betweenness centralities
#         betweenness_centralities = nx.edge_betweenness_centrality(graph.graph)
#         # Restore the graph
#         graph.graph = copy.deepcopy(graph_backup)

#     print(attacks.attacks)
#     print("Budget used : ", budget_used, budget, len(attacks.attacks))

#     return attacks

def betweenness_centralities_moving_attack(graph : CityGraph, nbTimes : int, budget : int, delta : int, costPerLane : bool) -> DynamicAttack :
    """
    We remove the edges with the highest betweenness centrality.
    Then we recompute the betweenness centralities for the edges of the new graph with the removed edges.
    """

    def remove_edge(graph : CityGraph, edge : tuple) :
        """Removes an edge from the graph"""
        graph.graph.remove_edge(edge[0], edge[1])

    costFunc = graph.get_nb_lanes_of_edge if costPerLane else lambda x : 1

    attacks = DynamicAttack(graph, nbTimes)
    betweenness_centralities = nx.edge_betweenness_centrality(graph.graph)
    graph_backup = copy.deepcopy(graph.graph)
    edges_deleted_last_time = []

    for time in range(1 + (nbTimes // delta)) :
        # Sort the edges by betweenness centrality of the last graph
        betweenness_centralities = list(betweenness_centralities.items())
        betweenness_centralities.sort(key = lambda x : x[1], reverse = True)
        # Append all the edges that were deleted last time, in case we have a budget of more than half of the number of edges
        for edge in edges_deleted_last_time :
            betweenness_centralities.append((edge, 0))

        # Remove all the edges with the highest betweenness centrality with the budget
        betweenness_centralities = [(x[0][0], x[0][1]) for x in betweenness_centralities]
        betweenness_centralities = [(graph.find_edge_index(x[0], x[1]), x[1]) for x in betweenness_centralities]
        budget_used = 0
        edges_deleted_last_time = []
        while budget_used < budget and budget_used < len(betweenness_centralities) :
            edge_idx = betweenness_centralities[budget_used][0]
            edge = graph.get_edge_at(edge_idx)
            attacks.add_attack(time, edge_idx)
            remove_edge(graph, edge)
            edges_deleted_last_time.append(edge)
            budget_used += costFunc(edge_idx)

        # Recompute the betweenness centralities
        betweenness_centralities = nx.edge_betweenness_centrality(graph.graph)
        graph.graph = copy.deepcopy(graph_backup)

    return attacks

# def index_attack(graph : CityGraph, nbTimes : int, budget : int, delta : int, costPerLane : bool) -> DynamicAttack :
#     """
#     We remove the edges with the highest betweenness centrality.
#     Then we recompute the betweenness centralities for the edges of the new graph with the removed edges.
#     """

#     def remove_edge_idx(graph : CityGraph, edge_idx : int) :
#         """Removes an edge from the graph"""
#         edge = graph.get_edge_at(edge_idx)
#         graph.graph.remove_edge(edge[0], edge[1])

#     attacks = DynamicAttack(graph, nbTimes)
#     graph_backup = copy.deepcopy(graph.graph)

#     for time in range(1 + (nbTimes // delta)) :
#         budget_used = 0
#         for i in range(graph.nb_edges()) :
#             if budget_used < budget :
#                 attacks.add_attack(time, i)
#                 remove_edge_idx(graph, i)
#                 budget_used += 1
#             else :
#                 break
#         graph.graph = copy.deepcopy(graph_backup)

#     return attacks

def connex_attack(graph : CityGraph, nbTimes : int, budget : int) -> DynamicAttack :
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

    attacks = DynamicAttack(graph, nbTimes)
    edges = list(graph.edges())

    # Find the bridge with the highest betweenness centrality
    # Then remove it
    for time in range(1 + (nbTimes // budget)) :
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

class TermColors:
    HEADER = '\033[95m'
    BLUE = '\033[94m'
    CYAN = '\033[96m'
    GREEN = '\033[92m'
    YELLOW = '\033[93m'
    RED = '\033[91m'
    ENDC = '\033[0m'
    BOLD = '\033[1m'
    UNDERLINE = '\033[4m'


NO_ARG = -1
import argparse

def main():

    # nom_ville : str, type_attack : str, times : int, delta : int (default 8), output_file : Optional[str]
    parser = argparse.ArgumentParser(description="Generate an attack on a city graph")
    parser.add_argument("nom_ville", type=str, help="The name of the city")
    parser.add_argument("type_attack", type=str, help="The type of attack, can be 'random', 'min_lanes', 'max_lanes', 'betweenness_centralities', 'moving' or 'connex'")
    parser.add_argument("times", type=int, help="The number of times the attack is repeated")
    parser.add_argument("budget", type=int, help="The budget of the attack")
    parser.add_argument("delta", type=int, help="The variation frequency of the attack")
    parser.add_argument("output_file", type=str, help="The output file", nargs="?", default=None)

    args = parser.parse_args()
    args.output_file = args.output_file if args.output_file is not None else f"{args.nom_ville}_{args.type_attack}_{args.times}_{args.budget}_{args.delta}"
    graph = CityGraph(args.nom_ville)
    if args.times == NO_ARG :
        # Convert the graph to be non oriented else networkx is mad >:(
        tmp_graph = None
        if graph.graph.is_directed() :
            tmp_graph = graph.graph.to_undirected()
        else :
            tmp_graph = graph.graph

    # TODO : Peut etre transformer en variable dynamiquement si ca devient trop
    # TODO : Random a l'air de pas marcher : meme avec un budget de 40000, il n'arrive pas a tout retirer
    switch = {
        "random" : random_attack,
        "min_lanes" : min_lanes_attack,
        "max_lanes" : max_lanes_attack,
        "betweenness" : betweenness_centralities_attack,
        "betweenness_moving" : betweenness_centralities_moving_attack,
        "random_moving" : random_moving_attack,
        "identifier" : identifier_attack,
        "longitude" : longitude_attack,
        #"connex" : connex_attack,
        "none" : no_attack,
    }

    if args.type_attack not in switch :
        print(f"{TermColors.RED}Error{TermColors.ENDC}: {args.type_attack} is not a valid attack type")
        return

    #print(args)

    # Write graph to file
    FILES_DIR_NAME = "data"
    diameter = graph.write_to_file(f"{FILES_DIR_NAME}/{args.nom_ville}/graph")
    if args.times == NO_ARG :
        args.times = diameter
    if args.budget > graph.nb_edges(): 
        args.budget = graph.nb_edges()

    # Write attack to file
    a = switch[args.type_attack](graph, args.times, args.budget, args.delta, False)
    a.set_delta(args.delta)
    a.write_to_file(f"{args.output_file}")

if __name__ == "__main__" :
    main()