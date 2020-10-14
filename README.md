# Rusty Vote

Rusty vote is a voting simulator built in Rust which simulates elections to determine the best voting strategy for a given voting system under certain assumptions to determine the optimal voting strategy. It uses GPU programming with Arrayfire to simulate a large number of elections quickly.

## Motivation

According to [wikipedia](https://en.wikipedia.org/wiki/Score_voting):
> Score voting or range voting is an electoral system for single-seat elections, in which voters give each candidate a score, the scores are added (or averaged), and the candidate with the highest total is elected.

Score voting is close to an ideal voting system when people vote honestly, as it allows for voters to express the value (or "utility") that they attribute to each candidate, and then the candidate with the highest value is selected. However people often vote strategically, which leads to different voting behavior:
> Ideal score voting strategy for well-informed voters is identical to ideal approval voting strategy, and a voter would want to give their least and most favorite candidates a minimum and a maximum score, respectively. 

This behavior takes away from one of the ideal aspects of score voting, which is its ability to represent the real values of voters. For example, voter might like a candidate less than one candidate but more than another, by some amount. If they are voting strategically, then they will assign this candidate either minimum or maximum value and the information that the candidate is their middle preference is lost.

The problem is essentially that certain possible ballots (where "ballot" represents the collection of scores given to each candidate by a voter) are inherently stronger than others, in particular the ballots which use all minimum and maximum values.

Under current consideration in this project are a class of voting systems that attempt to solve this problem by essentially "balancing" the inherent strength of ballots so that no possible ballot is inherently stronger than any other (under certain assumptions).

## Balanced Score Voting

The voting systems currently used in this project work as follows:
Determine a function to apply to a raw score voting ballot which outputs a value representing how inherently strong that ballot is. This value is then used as a divisor, dividing each score in the raw ballot by that value to produce the normalized ballot. This procedure is done for each ballot, and then the normalized ballot results are totaled to produce the winner.

The first system that was tested in this way used standard deviation as the function. This produced promising results, where a strategic voter with a medium preference candidate would not betray that preference by voting minimum or maximum, instead voting somewhere in the middle for that candidate. However, there was not a 1 to 1 correspondence between the voters preference and the optimal strategic vote, instead the voter still had some incentive to bias their middle vote toward the minimum of the score range. If all voters were to vote this way and take into account other voters behavior, this would eventually result in a [nash equilibrium](https://en.wikipedia.org/wiki/Nash_equilibrium) of [bullet voting](https://en.wikipedia.org/wiki/Bullet_voting).

An attempt at correcting this issue leads to the second system tested in this project, which is a modification of the first. Essentially, in the first voting system using standard deviation, middle-preference-candidate votes are biased toward the lower end so as to give one's favorite candidate more chance to win.  So we need to modify the divisor function (which was standard deviation in the first voting system) so that it is a bit smaller in the case where the middle vote is close to the maximum, and bigger when the middle vote is close to the minimum.

The way this is done to produce the second voting system is not so complicated: instead of using deviation from the mean in the standard deviation formula, we can use the maximum instead. This way as the vote values get further from the maximum, the normalized ballot will be weakened to some degree. 

The results of this second system are very good. The optimal strategy for a strategic voter ends up very close to the ideal of an honest vote in most cases.

Note that so far these systems have only been tested with elections 3 candidates and 3 voters. It is not clear at this time how changing the number of voters and/or candidates would affect the results.

## TODO

Add charts/graphs/data to give a clearer view of the results.

Make number of voters and number of candidates a parameter that can be modified and test with different values.

Investigate other functions which might produce a more balanced score voting system. 

## How to use

Install the version of arrayfire that matches the arrayfire crate version in `Cargo.toml`. Once you have rust and arrayfire, you should be able to run it without issue. If you want to change the way the simulation works, simply edit `main.rs` directly.

## Performance

On a machine using an Nvidia Geforce 2070 super, this can do 5,000,000 simulated elections (3 voters, 3 candidates) in under a second. This is at least an order of magnitude faster than an equivalent project done with python and tensorflow. It is much, much faster than an equivalent project done without parallelization.  

## License
[MIT](https://choosealicense.com/licenses/mit/)