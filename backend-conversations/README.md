# Conversations Application

**Updated:** 16-10-2025
**Author:** Farid Guzman

## Technical design

                                   +---------------------------+
                                   |  VoiceLog Processing      |
                                   |         A11/A18           |
                                   +-------------+-------------+
                                                 |
                                                 v
                                            +---------+
                                            |   SQS   |
                                            +---------+
                                                 |
                                                 v
                            +--------------------------------------+
                            |   conversations-api-feeder           |
                            +------------------+-------------------+
                                               |
                                 POST /conversation
                                               |
                                               v
   +--------------------+       +---------------------------+       +-----------------------+
   | LifeCycle Lambda   | <---> |        CloudMap           |       |      Discovery        |
   +--------------------+       +---------------------------+       +----------+------------+
                                                                            HTTP |
                                                                                 v
                                                          +--------------------------------------+
                                                          | Conversations API (Laravel / K8s)     |
                                                          +------------------+-------------------+
                                                          | GET /config                       |
                                                          | GET /conversations                |
                                                          | GET /conversations/<guid>         |
                                                          | POST /stream/<guid>/start         |
                                                          | GET /groups                       |
                                                          +------------------+-------------------+
                                                                  |                   |
                                                                  | SDP               | SDP
                                                                  v                   v
                                       +----------------------+     +---------------------------+
                                       | Streaming Microservice |    | conversations-api-v2     |
                                       |     (Pion GO)          |    |    (Kubernetes GO)       |
                                       +----------------------+     +---------------------------+
                                                |
                                                v
                       +------------------------+----------------------------+
                       |                    VoiceLogs S3                     |
                       +------------------------+----------------------------+
                                                |
                                                v
                                        +---------------+
                                        |  Postgres DB  |
                                        +---------------+


   ==================================================================================================

                       +----------------+        +------------------+        +------------------+
 DNS:                  | AWS ALB (mgr)  | -----> | Manager Web Srv  | -----> |   Keycloak        |
 ggd.myCompany.app      +----------------+        +------------------+        +------------------+
 tenant1.myCompany.app                                         ^                       |
 tenant2.myCompany.app                                          |                       |
                                                               |                       |
                                  +-----------------------------------+                |
                                  |     Keycloak Go Service          | <--------------+
                                  +-----------------------------------+
                                               POST /tenant


   ==================================================================================================

                                                    +-----------------------+
                                                    | Conversations Frontend |
                                                    |    (Vue.js, K8s)       |
                                                    +-----------+-----------+
                                                                |
                                                              Peer
                                                          (WebRTC/UDP)
                                                                |
                                                                v
                                                    +-----------------------+
                                                    |   Peer Connection     |
                                                    +-----------------------+


   ==================================================================================================

                           +-------------------------------------------+
                           | AWS Load Balancer (tenants)               |
                           |  myCompany.app/conversations/              |
                           +-------------------------------------------+
### EUEC1 / EC1 deployment

In the EUEC1 environment the conversations App in EC1 is used.
See the deployment diagram below the integration and dependencies.

## ERD

### Conversation
- destination (phone number / chatbox)
- tenant_guid
- started_at
- completed_at

### Participant
- display_name
- role (customer/agent)

### Participant Details
- address (phonenumber / ip?)

### Facts
- Conversation start
- Consult / Warm transfer
- Conference started
- Hold/ Unhold
- Switch / Takeback

### Fact Details
- Transfer destination

### Metadata
- Queue (id, name)
- Campaign (id, name)
- Strategy Type (Inbound / Outbound)
- Channel type (voice/chat)
- Filepath (including bucket)

### Groups
- display_name, can be nested to have multiple levels for example campaign → queue

### Tags
Picking up later, not required for TP

## Json example

```json
{
    "destination": "+31612345678",
    "tenant_guid": "0b816b4d898911eca00d02a7681a3548",
    "started_at": "2022-03-16 11:15:41.105",
    "completed_at": "2022-03-16 11:25:41.105",
    "participants": [
        {
            "display_name": "Bob",
            "role": "agent",
            "started_at": "2022-03-16 11:15:41.105",
            "completed_at": "2022-03-16 11:25:41.105",
            "details": [
                {
                    "key": "address",
                    "value": "tel:+31612345678"
                }
            ]
        },
        {
            "display_name": null,
            "role": "customer",
            "started_at": "2022-03-16 11:15:41.105",
            "completed_at": "2022-03-16 11:25:41.105",
            "details": [
                {
                    "key": "address",
                    "value": "tel:+31612345679"
                }
            ]
        }
    ],
    "facts": [
        {
            "type": "conversation_start",
            "started_at": "2022-03-16 11:15:41.105",
            "completed_at": null
        },
        {
            "type": "switch",
            "started_at": "2022-03-16 11:25:41.105",
            "completed_at": "2022-03-16 11:25:41.105",
            "details": [
                {
                    "key": "transfer_destination",
                    "value": "+31612345680"
                }
            ]
        }
    ],
    "metadata": [
        {
            "key": "campaign_id",
            "value": "1"
        },
        {
            "key": "campaign_name",
            "value": "Campaign Alpha 1"
        },
        {
            "key": "queue_id",
            "value": "1"
        },
        {
            "key": "queue_name",
            "value": "Queue Alpha 1"
        },
        {
            "key": "filepath",
            "value": "/charlie/999/2022-03-09/7/116.wav"
        },
        {
            "key": "strategy",
            "value": "inbound"
        },
        {
            "key": "channel_type",
            "value": "telephony"
        }
    ],
    "group": {
        "set": "campaigns",
        "display_name": "Campaign Alpha 1",
        "child": {
            "set": "queues",
            "display_name": "Queue Alpha 1"
        }
    }
}
```

## Authentication and authorization

### Current flow
Currently setup is as follows.

Backend provides configuration based on the requested hostname.
If the hostname is registered, the configuration response will contain:

```php
'display_name'    => $tenant->display_name,
'stun_servers'    => empty($stunServers) ? array() : explode(",", $stunServers),
'keycloak_url'    => $tenant->keycloak_url . '/auth',
'keycloak_realm'  => $tenant->keycloak_realm,
'keycloak_client' => $tenant->keycloak_client,
'sentry_dsn'      => config('sentry.dsn'),
'manager_url'     => $tenant->manager_url,
'quality'         => $tenant->quality,
```

The frontend ensures there is a keycloak token using the provided `keycloak_url`, `keycloak_realm` and `keycloak_client`.

The keycloak client has a mapper that collects user attributes from the keycloak user and adds a conversations attribute to the token.
This is a map with the tenant GUID as key and an object with the user GUID.

```json
{
  "email": "username@myCompany.com",
  "conversations": {
    "dcd5977277fd8cc63acf5f6d6d61e135": {
      "user_guid": "601d73e93652f6dc697a285843459166"
    },
    "69f84b5ffbcc9cbdae4e4c942f1f0da7": {
      "user_guid": "ea758d644c52b79deecf43df954bb9bc"
    },
    "ceee3c644015a910537036bf1a471698": {
      "user_guid": "9f3d51e17298a15ef4a86f35b92dd4ee"
    },
    "b5f09035cc6c99db5b25cb88369855f4": {
      "user_guid": "ba90e67fca76f04eb7aa2ce0daa6ad71"
    },
    "24359d7369b44952bfc8a8f2c3f890e8": {
      "user_guid": "ffd21f26637c45e6afb991a897fd2ce6"
    },
    "52c6e83862401bca9ebbcccc968e262e": {
      "user_guid": "641fbe484ea55ab4783dc938bed7fcd9"
    },
    "0b816d6b898911eca00d02a7681a3548": {
      "user_guid": "129ddc76ba4015a741f907"
    },
    "29552eea96d8e14fe6953446c3ffb27e": {
      "user_guid": "d3b615492f10c3a1dc6a8040c28e6634"
    },
    "c9ad8218f66547036be132688346d865": {
      "user_guid": "ea88ec34b6f4b2af0353a84e3e7d98a2"
    },
    "41d97bdbd8a6b9a922b2134b3c4ccb12": {
      "user_guid": "7ffa631de520c3dc8883f64884be2995"
    }
  },
  "family_name": "acme-admin",
  "given_name": "Floris",
  "typ": "ID",
  "sub": "1edff848-7682-4aa4-b5b5-ea5b07afe898",
  "aud": "conversations-app",
  "iss": "https://id.development.myCompany.dev/auth/realms/myCompanyapp",
  "jti": "d6ae1449-9ea0-4a25-9919-196084672781",
  "auth_time": 1748857307,
  "iat": 1748857309,
  "exp": 1748858209,
  "azp": "conversations-app",
  "nonce": "837c43f4-02cb-4562-870f-b679f495929d",
  "session_state": "71196ae7-05f4-48ff-862f-712716d2784b",
  "at_hash": "crRvRkYDISVHCQsMwpxYsA",
  "sid": "71196ae7-05f4-48ff-862f-712716d2784b",
  "email_verified": true,
  "name": "user name",
  "preferred_username": "username@myCompany.com"
}
```

The conversations API will validate the token on each API call.
It will call the realms endpoint on keycloak to retrieve the public key in order to validate the token.
The tenant ID for the request hostname should be present in the `conversations` attribute of the token.

When a user has access to multiple tenants, the token may get too large, causing the keycloak API to break due to HTTP request header that is too large.

The client will refresh the keycloak token keeping the Keycloak session alive.
Uses a hidden iframe.

### Proposed flow
Use OAuth PKCE flow.
This client library should enable the PKCE.
In addition, keycloak could enforce this on the conversations-app client.

When using the casbin-server authorization described below, all the required authentication information will be in the casbin server and only the user identity and tenant is needed to evaluate authentication.
The tenant can be derived from the hostname and the keycloak identity should be sufficient.
So there is no need to exchange the keycloak token with an application token and the current flow is sufficient.

### Using an application token

See also https://www.rfc-editor.org/rfc/rfc8693.html for OAuth 2.0 Token Exchange.
Two possible solutions:

#### Using a token service behind the conversations-api

+ No changes in the conversations-app front-end intergration with keycloak

+/- Token refresh handled by keycloak library and linked to keycloak session.

+ internal solution, can be changed without chances to the frontend. Migration issues not exposed to the front end

need a service per instance to get user metadata, but this can be pushed along with tenant information.

+ Disabling a keycloak account does immediately invalidate the conversation app session.

#### Using a public token service

+ No persistent keycloak session, reduces risk of session hijacking.
- Disabling a keycloak account does not immediately invalidate the conversation app session.
- Requires front end changes, and the existing keycloak-js library might not be sufficient.

#### Both solutions

+ No authorization information in keycloak
+ Token only needs to contain information for the current tenant.
+ Decreased complexity
+ Needs no updating user attributes with keycloak-api
+ No token mappers in keycloak
- Increased complexity for token services

## Authorization rules

Information required to do authorization:

- User identity (keycloak) (also used for auditing)
- Tenant GUID
- User GUID
- Campaign GUID (optional, should be used to restrict)
  - One or more?

Permissible actions:

- view/find calls and metadata
- playback
- download

Data restrictions:

- Always restrict to tenant
- Optional restrict to interactions
  - answered by the user
  - received on the restricted campaign(s)
  - Current implementation in call reports is restricting access to one campaign (Current campaign).

### Questions

**Q: Restrict one or more campaigns?**
- Decision: Proposed model supports multiple campaigns. Current manager configuration will only allow setting one campaign.

**Q: Are data restrictions mutually exclusive or not?**
So either all calls or own calls or calls from one campaign. Or own calls + calls from a campaign (might or might not overlap)
- Decision: Proposed model supports both sets restrictions.

**Q: Are campaign based data restrictions time based?**
If a user was associated with a campaign for a given period, only data pertaining to this period are accessible. If not, all data for any period will be accessible.
- Decision: Current configuration defines current permission set. This means that current access to historical data may be removed and previously inaccessible data may become accessible.

**Q: Are permissible actions linked to a data restriction or global?**
So, for instance can we want one specific user to:
- view all calls from the tenant.
- view and playback all calls in campaign (and if multiple campaigns are possible, with the same or different permissible actions)
- view, playback and download his own calls

Decision: will be part of the model. The model will not restrict this.

Based on decisions we can design the user permission data model to be used in enforcing authorization.

### Management of permissions


In the current setup we the following settings in the manager that affect the download and restriction on A11.

on tenant level we can enable/disable download of recordings. 

in order to view the report calls page the PAGE.PAGE_CALLS permission is needed.
based on the tenant level setting and the availability of a recording a download button may be shown.

The URL to download the recording only verifies the tenant setting. 
So regardless of the PAGE.PAGE_CALLS permission, the actual download recordings can be downloaded through the manager.

a user has ‘restrict view’ check box that determines if only calls of the users current campaign are visible in call reports.
Changing the campaign requires a separate permission PAGE_ELEMENT.PAGE_USER_ELEMENT_CAMPAIGN_ID

a user may have the ‘change campaign’ option. In checked the user may select a any campaign that has queues for which the user is skilled.
Currently it is possible that a user has both ‘restrict view’ and ‘choose campaign’ thus making it possible for the user to change the restricted campaign (depending on skills)
-> Might be good to prevent a user from having both settings enabled.

Restrict view is based on the current campaign of the queue the call was on. 
So changing the campaign on queue will change the access to historical calls on this queue for users with restrict view.
In the conversations app the campaign on which the call happend is fixed and will not change.
Changing the campaign on a queue will only affect new calls in the conversations app.

We need a place to manage roles/permissions.

## Casbin implementation

Using the casbin-server we can do live authorization checks efficiently.
The casbin-server can cache all policies in memory. The applications use a casbin client library doing grpc requests to the casbin server.
Both the Enforcer and Admin APIs are available in a grpc interface.
The policy database can be populated and kept up-to-date using a synchonization job that translates roles and permissions stored in the classic database into policies. Policies could also be populated from the ConfigService using a reconciler.

The drawing below also shows how campaigns can be turned in to “groups” or “roles” that can be enforced and retrieved using the Casbin APIs.



The tenant from interactions is mapped to the domain concept in casbin.
Permissions can be limited to specific services using the service value in the policy.
The domain concept is not a synonym for tenant in the classic context. A domain is the context for the policy. The config service can have it’s own domain, in that case the Casbin domain concept is not a interactions tenant but something bigger.


Here's a quick test in [Editor | Casbin](https://casbin.org/editor/)

### Model

```ini
[request_definition]
r = sub, dom, obj, act, svc
[policy_definition]
p = sub, dom, obj, act, svc
[role_definition]
g = _, _, _
g2 = _, _, _
[policy_effect]
e = some(where (p.eft == allow))
[matchers]
m = r.dom == p.dom && r.obj == p.obj && r.act == p.act && r.svc == p.svc && (g(r.sub, p.sub, r.dom) || g2(r.sub, p.sub, r.dom))
```

### Policy

```
p, alice, tenant1, data1, read, svc1
p, bob, tenant1, data2, write, svc1
p, ROLE_READER, tenant1, data3, read, svc1
p, ROLE_WRITER, tenant1, data3, write, svc1
p, CAMPAIGN_1, tenant1, data4, read, svc1
g, alice, ROLE_READER, tenant1
g, alice, ROLE_X, tenant1
g, ROLE_X, ROLE_WRITER, tenant1
g, charlie, ALL, tenant1
g, myCompanyapp/ff6b8df1-4d20-4b6a-9d5c-f23c45705bb5, alice, tenant1
g2, alice, CAMPAIGN_1, tenant1
g2, alice, CAMPAIGN_2, tenant1
g2, bob, ALL, tenant1
g2, ALL, CAMPAIGN_1, tenant1
g2, ALL, CAMPAIGN_2, tenant1
```

Line 13 shows a mapping from iss/sub to user, the issuer is the realm extracted from the url.

Line 18-20 show how we could create one list of all campaigns in a tenant and assign that to a user.

### Request (input to enforce)

```
alice, tenant1, data3, read, svc1
alice, tenant1, data4, read, svc1
alice, tenant1, data3, write, svc1
bob, tenant1, data4, read, svc1
charlie, tenant1, data4, read, svc1
myCompanyapp/ff6b8df1-4d20-4b6a-9d5c-f23c45705bb5, tenant1, data3, read, svc1
```

### Result

```
true Reason: ["ROLE_READER","tenant1","data3","read","svc1"]
true Reason: ["CAMPAIGN_1","tenant1","data4","read","svc1"]
true Reason: ["ROLE_WRITER","tenant1","data3","write","svc1"]
true Reason: ["CAMPAIGN_1","tenant1","data4","read","svc1"]
false
true Reason: ["ROLE_READER","tenant1","data3","read","svc1"]
```

### With Agent and campaign restrictions

#### Model

```ini
[request_definition]
r = sub, dom, obj, act, svc
[policy_definition]
p = sub, dom, obj, act, svc
[role_definition]
g = _, _, _
g2 = _, _, _
[policy_effect]
e = some(where (p.eft == allow))
[matchers]
m = r.dom == p.dom && (p.obj =='ALL' || ( p.obj=='AGENT' && r.obj.agent ==r.sub) || (r.obj.campaign==keyGet(p.obj,"CAMP/*"))) && r.act == p.act && r.svc == p.svc && (g(r.sub, p.sub, r.dom) || g2(r.sub, p.sub, r.dom))
```

#### Policies

```
p, ROLE_ALL_READER  , tenant1, ALL, read, svc1
p, ROLE_ALL_WRITER  , tenant1, ALL, write, svc1
p, ROLE_AGENT_READER, tenant1, AGENT, read, svc1 
p, ROLE_AGENT_WRITER, tenant1, AGENT, write, svc1 
p, CAMP_A_READER, tenant1, CAMP/CAMPAIGN_A, read, svc1
p, CAMP_A_WRITER, tenant1, CAMP/CAMPAIGN_A, write, svc1
g, alice, ROLE_ALL_READER, tenant1
g, alice, ROLE_X, tenant1
g, ROLE_X, ROLE_ALL_WRITER, tenant1
g, charlie, ROLE_AGENT_READER, tenant1
g, charlie, ROLE_AGENT_WRITER, tenant1
g, bob, ROLE_AGENT_READER, tenant1
g, bob, CAMP_A_READER, tenant1
g, bob, CAMP_A_WRITER, tenant1
```

#### Input

The `obj` in the request is a structured object with a `campaign` field containing the campaign guid and the `agent` field containing the interactions user guid.
The `sub` for the request would be the interactions user guid.

```
alice,   tenant1, {"campaign":"CAMPAIGN_A", "agent":"bob"    }, read,  svc1
alice,   tenant1, {"campaign":"CAMPAIGN_A", "agent":"bob"    }, write, svc1
charlie, tenant1, {"campaign":"CAMPAIGN_A", "agent":"charlie"}, read,  svc1
charlie, tenant1, {"campaign":"CAMPAIGN_A", "agent":"charlie"}, write, svc1
charlie, tenant1, {"campaign":"CAMPAIGN_B", "agent":"bob"    }, read,  svc1
charlie, tenant1, {"campaign":"CAMPAIGN_B", "agent":"bob"    }, write, svc1
bob,     tenant1, {"campaign":"CAMPAIGN_A", "agent":"charlie"}, read,  svc1
bob,     tenant1, {"campaign":"CAMPAIGN_A", "agent":"charlie"}, write, svc1
bob,     tenant1, {"campaign":"CAMPAIGN_A", "agent":"bob"    }, read,  svc1
bob,     tenant1, {"campaign":"CAMPAIGN_A", "agent":"bob"    }, write, svc1
bob,     tenant1, {"campaign":"CAMPAIGN_B", "agent":"bob"    }, read,  svc1
bob,     tenant1, {"campaign":"CAMPAIGN_B", "agent":"bob"    }, write, svc1
```

#### Result

```
alice,   tenant1, {"campaign":"CAMPAIGN_A", "agent":"bob"    }, read,  svc1
alice,   tenant1, {"campaign":"CAMPAIGN_A", "agent":"bob"    }, write, svc1
charlie, tenant1, {"campaign":"CAMPAIGN_A", "agent":"charlie"}, read,  svc1
charlie, tenant1, {"campaign":"CAMPAIGN_A", "agent":"charlie"}, write, svc1
charlie, tenant1, {"campaign":"CAMPAIGN_B", "agent":"bob"    }, read,  svc1
charlie, tenant1, {"campaign":"CAMPAIGN_B", "agent":"bob"    }, write, svc1
bob,     tenant1, {"campaign":"CAMPAIGN_A", "agent":"charlie"}, read,  svc1
bob,     tenant1, {"campaign":"CAMPAIGN_A", "agent":"charlie"}, write, svc1
bob,     tenant1, {"campaign":"CAMPAIGN_A", "agent":"bob"    }, read,  svc1
bob,     tenant1, {"campaign":"CAMPAIGN_A", "agent":"bob"    }, write, svc1
bob,     tenant1, {"campaign":"CAMPAIGN_B", "agent":"bob"    }, read,  svc1
bob,     tenant1, {"campaign":"CAMPAIGN_B", "agent":"bob"    }, write, svc1
```

### POC with casbin and permissions

https://gitlab.myCompany.io/floris.korbijn/casbin-conversations

## Related content

- Conversation Lifecycle Events - DevOps - Product Architecture
- Functional Migration - Conversations App - MyCompany - Product Management
- Chat event - MyCompany Documentation
- Chats data export - Data Team
- Interactions Release Schedule 2025 - Classic Interactions
- TelePerformance (GGD) - Projects, Consultancy & Support
