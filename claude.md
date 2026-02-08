# The most important part

All edits, suggestions, and responses must keep this in mind: This project is intended to help me, the programmer, understand Rust and game dev concepts. The primary goal is NOT to develop a game.

I am an experienced developer but have no experience in Rust. I have also never made a game. I will benefit from verbose comments and explanations for why components and entities are created the way they are. I will also benefit from hearing about best practices and other considerations, even if we don't implement them.

I want to LEARN to do these things.

# Helpful resources

Official bevy examples can be found in /bevy-examples. When in doubt, look at that directory for the correct way to do things.

# The project

This is a game meant to teach the fundamentals of Rust and of game development using bevy. It is simple in scope but explores a lot of different features that bevy supports.

Specifically, this is a betting game. A user has an army vs. an enemy army, and the army automatically fights. No input from the user can change things. Based on random chance, the armies will fight each other until the entire other side is defeated. The user, at the end of the fight, can choose to continue onwards and potentially earn much more money but at the risk of losing all their earnings so far if they lose the next fight. If the user chooses to end, they get their reward and can then upgrade their army for the next rounds.

# Common mistakes

I often use OOP thinking instead of ECS. I have worked extensively with TypeScript (OOP) and with Godot, which builds things as scenes and nodes. For example, when building a slime, I think of making a new slime node and then attaching all related things to it. This is very different from the ECS mindset and I will likely mess this up frequently. This is my weakest point. Be quick to correct me when I slide back into OOP mindsets instead of ECS. I need frequent reminders.
