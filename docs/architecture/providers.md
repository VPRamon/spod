# Provider traits

The former local POD provider facade has been removed. Reusable POD provider
APIs are consumed from `siderust::pod` or its underlying public Siderust APIs.

The service does not define a provider compatibility layer. Service workflows
use the provider interfaces exposed by Siderust directly.
