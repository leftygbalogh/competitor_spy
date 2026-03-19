# Prompt Log

Append each user prompt in chronological order.

## 2026-03-19

### Prompt 001
We are going to create an AI governance template to be used for all our future software projects as guide and guardrails. It will include my preferences for your communication style, how you may and may not act based on my instructions, all the various agent.md files so we have specific personas for a complete virtual software development team etc.

What would you recomments as an ideal file/folder structure for this?
What should be the first few documents that we want to add?

My initial preference are that you are not to chatty. Do not act over eagerly. Do not do more than I ask for. And if in doubt or something is unclear, you ask a series of questions one by one to clarify.

### Prompt 002
layout is fine,
yes I want to specify the discovery order

Let's start with the your instructions, here is my first attempt, please read it and suggest ways to improve it:
[User provided a long assistant-policy draft containing behavior rules, image handling notes, model family notes, and artifact usage guidance.]

### Prompt 003
I want our interactions from your side to be questions based when my instructions are not clear or ambiguous; I want you to ask no more than a dozen questions to clarify, finetune etc.

Also, we do not move forward until the current stage is done.
You do not make indepepndent decisions and start writing code until you are explicitly told to do so.

I want all my prompts in this project and in all the subsequent projects based on this template to be recorded, appanded to a prompts.md file.

Another rule, always use a memory.md file. And if I am idle for 5 minutes, save the current status in there.

If I am idle for 5 minutes, always save the laters chages, to code and governance alike. Save and Commit if idle for 15 minutes.

All new project should be initialised with a git repo.
all new projects based on this make a copy of this AI governance template folder, make it their specific ai governance xzy folder and gfo through the questions and answers for each document that needs to be specified or customized for the given project, and use it for records and reference.

for development we like TDD, domain driven design, XP principles overall.

add these to your rules as well

One document template that is missing if the project brief that contains all the functional specifications, requirements, nonfunctional expectations, etc, add that to the list.

another doc is the formal specifiction based on the requirements, and yet another doc based on the formal specs is a numbered itemised task list that the developement team agents can use to figure our the work order and record their progress.

And let's get stated. I accept your proposals, so create the initial versions as proposed and let's see what questions you have as we move along.

### Prompt 004
Is this something that if we document it, it can be inherited by the projects based on this template?
If the answer is yes, go ahead, but make it work on windows and linux as well, and keep it short and simple.
if the answer is no, then let's just document it.

### Prompt 005
The fact that you did not suggest the next step implies that I overguarded somewhere in your instructions - can you show me the sections where your tendency to suggest next steps is defined?

### Prompt 006
nope, let's keep them as they are, and let me learn to adjust.
what is the first document in our template? what is its status? what information is missing from it? Would you suggest a skeleton, only the high level areas that we should add and expand on?

### Prompt 007
go for it, and then lets start with a set of questions and answers for the first item.

Also, going forward, and add this to our rules as well, if I ask you multiple questions, make it a little task list and I would like us to go through them one by one. So you answer the first, or ask back to clarify. then ask if we can move onto the next, if so, tick off the first and onto the second we jump

### Prompt 008
fuck no, we are off to a wrong start if that is what is implied. this governance applies to everything in a project that is based on this template, this is the ultimate governance tool. This is where our fundamental values are that can be used to prioritise tasks or cut stalemate standoffs, clarify ambiguity

### Prompt 009
let's readjust your preferences, if we have an itemized list and we ticked of off, you are allowed to move onto the next item and we start working on it. Please record this as and where appropriate in your rulesystem and lets move on

### Prompt 010
no, I am okay for you to make a judgement call

### Prompt 011
yes

### Prompt 012
also, use a little more puctuation if your communication is a multiline more complex paragraph. Helps me understand.

with regards to your question, every task that you do as part of your routine: no need to ask for permission or to say you have done it. It is implied that as part of your job you do those things.

I would like you to ask for permission that may have legal implications, such as downloading a repo, or building third party code. Areas that may have negative consequences, such as you refactoring something without me asking for it. Decisions that have longterm ramifications: such as project direction or testing strategy. Decisions where we do not have an implicit understanding that they are your remit.

Is this clear enough or can you suggest better working for this with the same intent?

### Prompt 013
can you explain this stage to me with an example or two?

### Prompt 014
yes, this is good, lets keep it explicit

### Prompt 015
this is good

### Prompt 016
can you give an example, like maybe apply option 1 and option 2 to this very situation so I understand it better

### Prompt 017
1. and I'll object of I dont like ti

### Prompt 018
ok

### Prompt 019
I want this whole template to be  a living improving, organically growing document that is continually refined, this is why I want each new project to copy the contents of this master document to its own local projectxyzgovernance folder, so if there is a broken step or an improvement I can just tell that agent to add it to the master template and start using it from there on with all subsequent projects. So we can start with a few examples, but this is going to grow and change as we implement more and more projects

### Prompt 020
good

### Prompt 021
One thing we need to factor in, and I need your suggestions on this, is that greenfield and brownfield software project work very differently. On a greenfield, I need to create new expert domain specific agents who can help me refine the scope and content of the project. Then good architects who can anticipate how the initial scope can evolve later. Good guys who can write specification, who can write pseudocode and formal mathenmatical specification and then developers who can implement it as specified.

On a brownfield project where we take an existing codebase and refactor it or rewrite it in a new language, I need people who can guard, test and describe the exiting functionality in such a way that we have the same exact functionality, not as documented, but as implemented. People who can take a big project, chunk it up to small tiny bits and work in thousands of tiny iterations and function by function, class by class, module by module rewrite it.

How do you suggest we factor this in? And in lihgt of this, we can go back to the previous docs if we need to, that is okay, part of natural evolution

### Prompt 022
go

### Prompt 023
I want you to give me a wider range than just my initial example were; keep the languages; but envisage the scenario where ai subagents need to do TDD in a classic pair-coding fashion as in XP, and then reviewer subagents look at it through the lens of our ultimate values: readability, and long term maintainability. We are writing this code for humans to look at, understand, maintain, improve upoon. So we need to implement everything that can help that from human readable specification to formal mathematical specification, to clear, structured code with no code smell, to proper documentation, everything is 100% test covered and not only for happy path but with an extreme defensiveness in mind where applicable. with documentation that has live code samples... so I want you to propose a large number of clearly scoped agents that need to work together, while each focuses on their speciality, and brings it, they can actually deliver results.

Does tht make sense and what are your suggestions?

### Prompt 024
perfect

### Prompt 025
are there any other langages that you think would warrant to be added?

### Prompt 026
recommendations accepted, implement them all as proposed

### Prompt 027
I would also like to throw in a bit of personality differences so we do not end up with indecision or procrastination when multiple agents work together so maybe there are two outcomes of this: a clear chain of command for each stage; and also a different developer personas who work well together but there is enough debate to generate the best solution based on the information available.

Would you be able to factor that into your variants as well?
And yes, please go ahead, implement your suggestions if you can factor in my suggestoin here

### Prompt 028
rust, this will be our main language

### Prompt 029
go

### Prompt 030
okay

### Prompt 031
no, this is enough. Lets add a few for our secondary langiage which is python and then the rest of the langiages should only have the main 12 or so. Implement all of these and then let me know what the next stage is for us

### Prompt 032
yes

### Prompt 033
green

### Prompt 034
I think we are now in a meta meta mode here. This project is about creating an ai template that we can use later on for greenfieald nad brownfield software projects; but when we use this template, they will need to go through this exact same discovery process as well. are we on the same page on this?

### Prompt 035
Name: AI Governance Template
Problem statement: Currently, I need to start from almost scratch for every software project that I have with my AI framework. I want to be able to move away from this and adopt an existing customised framework that works from me, so we spend less time on making the AI framework configured and more on the actual codiing projects.

### Prompt 036
I can take this folder, drop it in an emptry sofware project folder, tell the AI agent to read the contents of this folder and act accordingly. and then it says, excellent, I read it, I understand my limitations policies, so let's get to work, let me ask you questions about what kind of a software project we will be working on

### Prompt 037
1 is comparative metric, we will never test this like this that we run you twice in the same instructions - so out of the question. 2. subjective. all questions are useful if their intent is to clarify the situation and help make a decision and carry out the right action. 3. depends on the project size. maybe 10 maybe 1000.

Here is a metric - the number of times that I ask the AI agent in the software project to modify the original master template should decrease with each iteration. Each time I start a new project, I should feel less and less compelled to tweak the governance. The first time I expect 20 changes, second time a little less... after 10 projects, none at all for many projects to come, and then one or two as we discover a better way to work.

### Prompt 038
this is fucking beautiful. Let's make this as our master list; but lets also prioritise them into an urgency / importance grid so we start with the most important urgent ones, carry on with the important ones and come back to everything else when the actual given project requires it. is this doable?

### Prompt 039
perfect, please updates all docs and processes so we can follow this approach from now on

### Prompt 040
hang on. [re item 9] If I give you a greenfield project: build a CLI snake game. I want a formal specification, so that if I duplicate the project into two with that documentation but without any code implementation, and then I ask one project to implement it in rust and another to do it in bash, I get functional equivalents in terms of behaviour and functionality. The look and feel might be different, but they are both snake games, with the same configuration parameters, with the same algorithmic flows. But I do not need this algo flow to be mathematically provably correct. Does that make sense - can you summarize it with your own words to make sure I said what I meant?

### Prompt 041
hang on. If there is a good enough mathematical tooling that can provide solid scaffolding for the behavioural spec rigor, what would that tooling be?

### Prompt 042
go ahead

### Prompt 043
one more thing - is there a way for the actual coders to record how they implemented what they implemented as a third layer of documentation. Imagine the scenario, where the all the code is lost, but we have all these project documents, from commander's intent to specification and the implementation detail that they wrote down when they created the code, and we could take all of these and rerun the coding, saying please now just take the third layer of docs for your implementation guidelines and write the code as specified there.

### Prompt 044
yes

### Prompt 045
looks good, qq: do the personas have a clear definition of their duties, rights and obligations? What I have in mind is do the developers know that they need to record the implementation decisions and details for example and where they need to record it?

### Prompt 046
do we need to add a definition of ready for example for tickets and a definition of done as well at the various stages?

### Prompt 047
now
