#Tracing - your println 2.0.

This article aims to introduce the concept of tracing and relevant tools.

1. What do I mean by tracing?

Tracing provides ability for interested people to look into details of program internals and extract usable informations
while the program is working, without changing its code or configuration, without restarting it, with minimal influence on performance
and without any risk of causing problems. Those features are crucially important for debugging problems in environments that
cannot stand a downtime, or are not under control of person performing debugging. Specifically I am referring to tools
like Dtrace and BPF, or ETW for Windows.

Q: But I can just print what I want, what's wrong with that?

A: As an author of your application, you indeed can, and it may be the best way for you to fix the problem,
if you are using your computer.
However tracing is oftern performed in environments you don't control (ie. your customers servers),
by people who do not have access to source code, skills and confidence to modify it, or even tools to do it
(production environments often may not have compilers installed), under pressure of time. Also some problems
happen only in partcular circumstances, and are hard to reproduce, so restarting your application
may cause the problem to go away, without giving you a chance to fix it.


Q: But there are logs, don't they provide enough information?

A: Logs are usuefull and you should have them, but they are often insufficient to diagnose problems.
First of all, not all programs allow to modify what information is logged without restart.
This leads to conservatime logging most of the time, and a need to restart the application if there is specific problem.
Second, logs contain information you spefically expect to be useful, and thus log statement are typically rather rare.
In contrast, tracing is expected to be applied in many places in your code, even if the chance of it being used is one in a million,
because the cost of doing so is minimal, and when you look for source of particular problems, anything could be responsible.

Third, people using your application have no way of deciding where do you insert log statement.

Another problem with logs is that they require exporing data out of your application. With high-speed networks, any
attempt to print every data packet will fail miserably. Tools like BPF and DTrace however allow you to
run code generating summary of things that interest you and only export that. You can easily count
how many certain types of packets are sent, inpecting all of them, and only exporting the counter.
