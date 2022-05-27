## The patterns

Command-Query Responsibility Segregation (CQRS) and event sourcing are patterns that enable many of the concepts behind 
Domain Driven Design.
All of these tools are designed to provide a great deal of flexibility for applications that have complex or rapidly
changing business rules.

By separating the business rules from the technical aspects of an application we remove many
of the inherent barriers to software changes that exist in standard applications.
Any application with complex or rapidly change rules might be a good candidate for using CQRS and event sourcing.

### A note on terminology

Though CQRS and event sourcing can be used for a range of software problems they are primarily applied to build 
business applications since those so often require that quality.

I'll use the term "business rules" whenever I'm specifically discussing these complex or changing rulesets.
If your application is not a business application, just replace "business rules" with something more appropriate to your
domain.