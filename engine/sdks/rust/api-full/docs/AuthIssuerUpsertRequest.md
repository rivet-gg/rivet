# AuthIssuerUpsertRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**algorithm** | **String** | Pinned signature algorithm: EdDSA, RS256, or ES256. | 
**audience** | **String** | Required, exact `aud` claim for Rivet (never optional). | 
**grant_ceiling** | [**Vec<models::AuthIssuerGrant>**](AuthIssuerGrant.md) | Maximum access this issuer may embed in a signed Rivet JWT. | 
**issuer** | **String** | Exact `iss` claim to trust for this namespace. | 
**jwks_url** | **String** |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


