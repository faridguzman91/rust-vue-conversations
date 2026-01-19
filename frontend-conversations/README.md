# Vue 3 + TypeScript + Vite

This template should help get you started developing with Vue 3 and TypeScript in Vite. The template uses Vue 3 `<script setup>` SFCs, check out the [script setup docs](https://v3.vuejs.org/api/sfc-script-setup.html#sfc-script-setup) to learn more.

Learn more about the recommended Project Setup and IDE Support in the [Vue Docs TypeScript Guide](https://vuejs.org/guide/typescript/overview.html#project-setup).


# Conversations App - User Manual

**Updated:** 12-12-2025
**Maintained by:** Farid Guzman

---

## Table of Contents

- [Introduction](#introduction)
- [Authentication](#authentication)
- [Call Data Scope](#call-data-scope)
  - [Included Calls](#included-calls)
  - [Excluded Calls](#excluded-calls)
- [Retention Policy](#retention-policy)
  - [Standard Retention Period](#standard-retention-period)
  - [Exceptions](#exceptions)
  - [Call Recordings](#call-recordings)
- [Accessing Historical Data](#accessing-historical-data)
- [Filtering Calls](#filtering-calls)
  - [Filter Criteria](#filter-criteria)
- [Search Results Grid](#search-results-grid)
  - [Columns in the Grid](#columns-in-the-grid)
  - [Action Buttons](#action-buttons)
- [Audio Player](#audio-player)
- [Terminology](#terminology)
- [Date-Time Information](#date-time-information)

---

## Introduction

The Conversations App is a versatile and user-friendly tool designed to help you efficiently search, filter, and manage calls. Its intuitive interface includes a robust search function, a detailed results grid, and a built-in audio player for streaming call recordings. Users can take various actions on calls, such as reviewing call details, initiating quality reviews, playing back calls directly within the app or downloading the call recordings.

This manual provides instructions on:

- The types of calls available in the app.
- Filtering and searching calls using various criteria.
- How to interpret and use the grid displaying search results.
- Performing actions like viewing details, starting a review, playing back calls or download the audio recordings.

QuandaGo system requirements apply: [System Requirements](https://docs.google.com/document/d/1a0WzBNgQNrJDclRNcjkb4jH6MPj3r99hn44CyVX67_w/edit?tab=t.0)

---

## Authentication

**Login:** Use your assigned credentials to access the app. A successful login grants access to all authorized features.

**Logout:** Click the menu located in the top-right corner to securely log out.

---

## Call Data Scope

The call information is published after the call ended and therefore it can take up to a few minutes until it becomes available in the Conversations App.

The Conversations App provides access to all customer interactions which reached a queue. Please take note of the following inclusions / exclusions:

### Included Calls

- All calls successfully answered by an agent.
- Calls abandoned after they entered a queue (while queued, before being routed to an agent). For abandoned calls, the following is expected:
  - The Agent will be blank.
  - The Duration will display as `00:00:00`.
  - Time (Answered / Abandoned) will represent the time when the call was abandoned.
  - All recording related functions (e.g. playback, download) will be disabled / missing.

### Excluded Calls

- Calls that ended in the IVR (Interactive Voice Response) before being queued.
- Calls originated by the Predictive Outbound Dialer, rejected by the remote party.

---

## Retention Policy

### Standard Retention Period

Data within the Conversations App is retained for a default period of **90 days**. After this period, it is automatically removed from the application.

### Exceptions

#### Call Recordings

The retention policy is superseded when a call is associated with a media recording that is governed by a separate, extended retention policy. In such instances, the call record and its associated metadata will persist in accordance with the retention schedule of the call recording.

### Accessing Historical Data

Data that has exceeded the 90 days retention period is archived to the Data Lakehouse. Please contact your Business Intelligence (BI) team to request a specific data report, or query the Data Lakehouse directly.

To obtain direct access credentials for the Data Lakehouse, users must contact their designated organizational administrator.

---

## Filtering Calls

You can filter the list of conversations to find specific results based on your criteria. To filter the list:

1. Select your desired filter criteria from the available options.
2. Click the **Apply Filter** button to update the results.
3. To clear all active filters, click the **Reset** button.

> **Note:** The results grid does not update automatically as you change the filters. You must click Apply Filter to see your changes.

### Filter Criteria

The available filters include:

#### GUID

Enter a specific conversation identifier to retrieve its details.

> **Note:** If a value is provided in this field, all other filters will be ignored and reset to their default values.

#### From and To

Specify a time range using the date-time picker.

- The default interval is the last 24 hours.
- Up to 31 days of data can be queried at a time.
- Calls outside the selected range are excluded.

#### Agent

Enter part or all of an agent's name to search for calls assigned to that agent.

- Partial matches are supported.

#### Remote Party

Enter the full caller's (inbound) or callee's (outbound) phone number.

- Partial matches are not supported.

#### Queues

Select one or more queues from a drop-down with grouped check-boxes. Queues are organized by their associated campaigns.

#### Type

Select the interaction type from the predefined list.

#### Transfer Destination

Used to filter the calls transferred to a specific destination.

- Similar requirements as the Remote Party field.

---

## Search Results Grid

The search results are displayed in a scrollable grid with the following features:

- **Infinite scrolling:** All the results are displayed in a single view. The list is extended with more results (if available), as soon as the last item was displayed.
- **Sortable columns:** Click column headers to sort the data in ascending or descending order.

### Columns in the Grid

Each row in the grid represents a single call, with the following columns:

#### GUID

A unique, immutable identifier for the call.

- This column is not sortable.

#### Campaign

The name of the campaign the call was part of.

#### Queue

The name of the queue where the call was handled.

#### Agent

The name of the agent who handled the call.

- Empty - if the call was abandoned (customer hanged up while waiting in queue).
- Can contain names of automated system agents.

#### Remote Party

Depending on the call direction, the caller's (for inbound) or callee's (for outbound) phone number or identifier. The associated icons are offering a visual clue helping to quickly identify the call direction and result:

**Successfully connected calls:**
- **Inbound** - marked by a green icon with an arrow pointing down-left.
- **Outbound** - marked by a blue icon with an arrow pointing up-right.

**Unsuccessful calls:**
- **Inbound** - marked by a red icon with an arrow pointing down-left.
- **Outbound** - marked by a red icon with an arrow pointing up-right.

#### Time (Answered / Abandoned)

The date-time when the call was answered by an agent (local time).

- Formatted as `DD-MM-YYYY HH:MM` - with hours displayed in 24h format

#### Duration

The duration of the call, displayed in `HH:MM:SS`. This detail also reflects the duration of a recording, if present.

- For abandoned calls, this is always `00:00:00`.

#### Action Buttons

See the [Action Buttons](#action-buttons) section below.

---

## Action Buttons

When hovering over a row in the grid, a list of action buttons is displayed for that row. These buttons enable specific operations on the call.

### Details Button

Clicking on the Details button will open a modal containing detailed information about the conversation, containing the conversation GUID in its title.

**Displayed data:**

- **Campaign:** The name of the campaign the call was part of.
- **Queue:** The name of the queue where the call was handled.
- **Type:** The type of the call (e.g. Inbound Call, Outbound Call). The associated icons are offering a visual clue helping to quickly identify the call direction and result:
  - **Successfully connected calls:**
    - **Inbound** - marked by a green icon with an arrow pointing down-left.
    - **Outbound** - marked by a blue icon with an arrow pointing up-right.
  - **Unsuccessful calls:**
    - **Inbound** - marked by a red icon with an arrow pointing down-left.
    - **Outbound** - marked by a red icon with an arrow pointing up-right.
- **Call Received:** The time when the call was received.
- **Agent:** The name of the agent who handled the call.
  - `- -` if the call was abandoned (customer hanged up while waiting in queue).
  - Can contain names of automated system agents.
- **Call Answered:** The time when the call was answered.
- **Waiting Time:** The duration between the time when the call was queued and the time when it was answered (or abandoned).
- **Call Ended:** The time when the call was ended.
- **Transfer Destination:** If the call was transferred, this field will contain the receiving party phone number or identifier.
- **Completion Reason:** The reason why the agent was removed from the call - e.g. Hang Up, Leaving Conference, etc.
- **Recording:** Depending on the recording status this can be one of the following (dates in UTC):
  - `Available Until DD/MM/YYYY HH:MM:SS` - The recording is currently available and will be removed at the specified date.
  - `Expired At DD/MM/YYYY HH:MM:SS` - A recording existed, but was removed (as configured) at the specified date.
  - `Not Recorded` - The call was either abandoned in the IVR (Interactive Voice Response) or it was not configured to be recorded.
- **Customer Interaction GUID:** Links related calls from a single customer's interaction.
- **Call ID:** An internal ID for the call (not guaranteed to be unique or immutable).

### Download Button

The Download button is displayed only for calls that were configured to be recorded, and only if the current user has sufficient rights to download call recordings. When clicking the Download button, the audio recording of the call starts downloading on your local machine.

#### Recording File Naming Format

Downloaded recording files are automatically named using the following format:

```
[UTC date-time]-[agent name]-[conversation identifier].[extension]
```

While the UI displays the recording time in your local timezone, the filename always uses UTC for consistency across systems.

### Review Button

Displayed only if Quality Review is enabled for the tenant - when clicked, it opens the Quality Review Form in the Manager.

### Play Button

Clicking the Play button will stream the recording, using the built-in Audio Player.

**Conditions for availability:**

- The call has to be recorded and the recording must not be expired.
- The recording was processed and published after the call ended.

See [Network setup for WebRTC soft phone](https://docs.google.com/document/d/1a0WzBNgQNrJDclRNcjkb4jH6MPj3r99hn44CyVX67_w/edit?tab=t.0) if encountering problems with streaming the audio.

---

## Audio Player

The Audio Player is located at the bottom of the screen.

It includes a progress bar for tracking, and buttons (play / pause, stop, restart) to control the playback. When a track finishes the user can restart playback from the beginning. The player is reset on pressing the stop playback button.

---

## Terminology

- **Call/Interaction:** Currently synonymous as the app supports only phone calls (voice interactions). In the future, other interaction types (e.g. chat) may be added.
- **Inbound Abandoned Call:** A call from a customer who disconnected before the agent was able to speak to the customer.
- **Agent Interaction:** The communication of an agent with other participants from connection till disconnection. Currently the Conversations App contains only voice interactions of agents (and abandoned calls).
- **Customer Interaction:** The communication of the customer with other participants (including IVR, waiting music and Bots) from connection till disconnection. When the customer is transferred to another agent, a Customer Interaction can contain multiple Agent Interactions, linked together by the Customer Interaction GUID.
- **Filtering/Search:** Used interchangeably to describe narrowing down results.

---

## Date-Time Information

Display local date-time based on local device timezone.

---
