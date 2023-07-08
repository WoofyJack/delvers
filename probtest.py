import random

active_delver = 0.7
other_delvers = [0.3, 0.3, 0.3]

defender = 0.6

active_amount = 1
team_amount = 0.25

delvers_stats = (active_amount+team_amount)*active_delver+sum(i*team_amount for i in other_delvers)
print("Delver Stats:", delvers_stats)

n = 10000
wins = 0
for i in range(n):
    delver_roll = random.random() * delvers_stats
    defender_roll = random.random() * defender
    if delver_roll > defender_roll:
        wins += 1
    # print(delver_roll, defender_roll, delver_roll > defender_roll)
print(wins/n)