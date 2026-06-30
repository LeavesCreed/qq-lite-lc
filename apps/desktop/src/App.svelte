<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  type ConversationView = {
    id: string;
    title: string;
    last_message_preview?: string;
    updated_at_ms: number;
    unread_count: number;
  };

  type RichNode =
    | { Text: { text: string } }
    | { Unsupported: { kind: string; summary: string } };

  type MessageView = {
    local_id: string;
    conversation_id: string;
    sender_id: string;
    direction: "Incoming" | "Outgoing";
    nodes: RichNode[];
    send_state: unknown;
  };

  let endpoint = "ws://127.0.0.1:3001";
  let token = "";
  let status = "idle";
  let conversations: ConversationView[] = [];
  let selectedConversationId = "";
  let messages: MessageView[] = [];
  let composer = "";

  onMount(async () => {
    await refreshConversations();
  });

  async function connect() {
    status = "connecting";
    await invoke("connect", { endpoint, accessToken: token || null });
    status = "connected";
  }

  async function refreshConversations() {
    conversations = await invoke<ConversationView[]>("list_conversations");
    if (!selectedConversationId && conversations.length > 0) {
      selectedConversationId = conversations[0].id;
      await refreshTimeline();
    }
  }

  async function selectConversation(id: string) {
    selectedConversationId = id;
    await refreshTimeline();
  }

  async function refreshTimeline() {
    if (!selectedConversationId) return;
    messages = await invoke<MessageView[]>("list_messages", {
      conversationId: selectedConversationId
    });
  }

  async function send() {
    const text = composer.trim();
    if (!selectedConversationId || !text) return;
    await invoke("send_text_message", {
      conversationId: selectedConversationId,
      text
    });
    composer = "";
    await refreshTimeline();
  }

  function renderNode(node: RichNode) {
    if ("Text" in node) return node.Text.text;
    return node.Unsupported.summary;
  }
</script>

<main class="shell">
  <aside class="sidebar">
    <div class="connection">
      <input bind:value={endpoint} aria-label="NapCat WebSocket endpoint" />
      <input bind:value={token} aria-label="NapCat access token" placeholder="access token" />
      <button on:click={connect}>Connect</button>
      <span>{status}</span>
    </div>

    <nav class="conversation-list" aria-label="Conversations">
      {#each conversations as conversation}
        <button
          class:selected={conversation.id === selectedConversationId}
          on:click={() => selectConversation(conversation.id)}
        >
          <strong>{conversation.title}</strong>
          <span>{conversation.last_message_preview ?? ""}</span>
        </button>
      {/each}
    </nav>
  </aside>

  <section class="chat">
    <div class="timeline">
      {#each messages as message}
        <article class:outgoing={message.direction === "Outgoing"} class="message">
          <div class="sender">{message.sender_id}</div>
          <div class="bubble">
            {#each message.nodes as node}
              <span>{renderNode(node)}</span>
            {/each}
          </div>
        </article>
      {/each}
    </div>

    <form class="composer" on:submit|preventDefault={send}>
      <input bind:value={composer} placeholder="Type a message" />
      <button type="submit">Send</button>
    </form>
  </section>
</main>

<style>
  .shell {
    display: grid;
    grid-template-columns: 320px 1fr;
    height: 100vh;
    overflow: hidden;
  }

  .sidebar {
    display: grid;
    grid-template-rows: auto 1fr;
    border-right: 1px solid #d9dde3;
    background: #ffffff;
    min-width: 0;
  }

  .connection {
    display: grid;
    gap: 8px;
    padding: 12px;
    border-bottom: 1px solid #e5e8ee;
  }

  .connection input,
  .composer input {
    width: 100%;
    border: 1px solid #cdd3dc;
    border-radius: 6px;
    padding: 8px 10px;
    background: #ffffff;
  }

  .connection button,
  .composer button {
    border: 0;
    border-radius: 6px;
    padding: 8px 12px;
    color: #ffffff;
    background: #2f6f6d;
    cursor: pointer;
  }

  .conversation-list {
    overflow-y: auto;
  }

  .conversation-list button {
    display: grid;
    gap: 4px;
    width: 100%;
    padding: 12px;
    border: 0;
    border-bottom: 1px solid #eef0f3;
    text-align: left;
    background: transparent;
    cursor: pointer;
  }

  .conversation-list button.selected {
    background: #e9f2f1;
  }

  .conversation-list span {
    color: #68717d;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .chat {
    display: grid;
    grid-template-rows: 1fr auto;
    min-width: 0;
  }

  .timeline {
    display: flex;
    flex-direction: column;
    gap: 12px;
    overflow-y: auto;
    padding: 20px;
  }

  .message {
    max-width: min(680px, 82%);
  }

  .message.outgoing {
    align-self: flex-end;
  }

  .sender {
    margin-bottom: 4px;
    color: #68717d;
    font-size: 12px;
  }

  .bubble {
    border-radius: 8px;
    padding: 10px 12px;
    background: #ffffff;
    border: 1px solid #e0e4ea;
    line-height: 1.5;
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }

  .outgoing .bubble {
    background: #dff0ed;
    border-color: #c5e1dc;
  }

  .composer {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 10px;
    padding: 12px;
    border-top: 1px solid #d9dde3;
    background: #ffffff;
  }
</style>
