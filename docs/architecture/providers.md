# Provider traits

The former local POD provider facade has been removed. Reusable POD provider
APIs are consumed from `siderust::pod` or its underlying public Siderust APIs.

The service keeps only the small `spod::service::providers::EphemerisProvider`
trait needed by its SPICE/LISA artifact adapters; it is not a scientific
compatibility layer.
