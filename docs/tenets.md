# Our Tenets

These tenets are shared goals that we keep in mind when building this family of projects.
Together they provide a basis for making design decisions when multiple solutions present themselves.

### Target serverless but ensure support for standalone applications.
> Serverless solutions are the primary use case for this project, but in this pursuit we should sacrifice neither
> features nor performance when used in standalone applications.

### Be lightweight - prefer distributed over monolithic solutions.
> Our target architectures are serverless functions or microservices: lightweight, likely distributed and decoupled.
> To fit these needs we strive for reduced startup and processing latency, a smaller memory footprint and minimal 
> complexity. We deprioritize the more full-featured but cumbersome approach that is characteristic of many 
> CQRS frameworks.

### Avoid unsafe code.
> Rust provides an exceedingly fast and safe environment which we should not take for granted.
> Unsafe code must be heavily tested and meticulously maintained to ensure the Rust contract of safety is not broken,
> that being the case unsafe code should be avoided wherever possible.

### Enforce Domain Driven Design principals.
> This package is focused on DDD solutions using CQRS and event sourcing rather than a general purpose event processing
library. We should therefore enforce the guidelines and best practices around those topics in an opinionated way. 
