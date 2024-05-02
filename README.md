# LinkStream Robustness Tool

This project was made during the course "Research Project" at Sorbonne University in the 3rd year of license.
The goal is to measure a unit called efficiency on a linkstream.

# Different parts of the project

- The first part is generating graph data, we use OSMnx to generate a graph from a city after some data normalisation.
- The second part is attacking the graph by removing links according to different strategies over time.
- The third part is to measure the efficiency of the graph after the attacks.

# How we approached the problem

We decided to discretise the time in the linkstream, thus making it a time-varying graph.

# How to use the project

You can find the many ways to launch an attack in the scripts folder.