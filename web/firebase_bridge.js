// Firebase Bridge for Camel Up Multiplayer
// This file provides JavaScript functions that can be called from Rust via wasm-bindgen

import { initializeApp } from 'https://www.gstatic.com/firebasejs/10.7.1/firebase-app.js';
import {
    getDatabase,
    ref,
    set,
    push,
    onValue,
    get,
    update,
    remove,
    serverTimestamp,
    off
} from 'https://www.gstatic.com/firebasejs/10.7.1/firebase-database.js';
import {
    getAuth,
    signInAnonymously,
    onAuthStateChanged
} from 'https://www.gstatic.com/firebasejs/10.7.1/firebase-auth.js';

// Firebase instances
let app = null;
let db = null;
let auth = null;
let currentUserId = null;

// Generate a unique session ID for this tab (to distinguish multiple tabs with same Firebase auth)
const tabSessionId = crypto.randomUUID();

// Store active listeners for cleanup
const activeListeners = new Map();

// Queues for receiving data from Firebase (polled by Rust)
window.firebaseGameStateQueue = [];
window.firebaseActionsQueue = [];
window.firebasePlayersQueue = [];
window.firebaseAuthReady = false;
window.firebaseError = null;

// Initialize Firebase with the provided config
window.initializeFirebase = function() {
    const config = {
        apiKey: "AIzaSyBZ4zFjkHLyz1tA_q_-dJCwStYOtknzkyw",
        authDomain: "camel-up-4747b.firebaseapp.com",
        databaseURL: "https://camel-up-4747b-default-rtdb.asia-southeast1.firebasedatabase.app",
        projectId: "camel-up-4747b",
        storageBucket: "camel-up-4747b.firebasestorage.app",
        messagingSenderId: "608825613332",
        appId: "1:608825613332:web:90111ed0a101d9c3abdb01"
    };

    try {
        app = initializeApp(config);
        db = getDatabase(app);
        auth = getAuth(app);
        console.log('Firebase initialized successfully');
        return true;
    } catch (error) {
        console.error('Firebase initialization error:', error);
        window.firebaseError = error.message;
        return false;
    }
};

// Sign in anonymously
window.signInAnonymously = async function() {
    try {
        const result = await signInAnonymously(auth);
        // Use tab session ID to allow multiple tabs with same Firebase auth
        currentUserId = tabSessionId;
        window.firebaseAuthReady = true;
        console.log('Signed in anonymously, tab session:', currentUserId);
        return currentUserId;
    } catch (error) {
        console.error('Sign in error:', error);
        window.firebaseError = error.message;
        return null;
    }
};

// Get current user ID
window.getCurrentUserId = function() {
    return currentUserId;
};

// Check if Firebase is ready
window.isFirebaseReady = function() {
    return app !== null && db !== null && auth !== null;
};

// Check if authenticated
window.isAuthenticated = function() {
    return currentUserId !== null;
};

// Create a new room
window.createRoom = async function(roomCode, hostName, characterId, colorIndex) {
    if (!currentUserId) {
        console.error('Not authenticated');
        return false;
    }

    try {
        const roomRef = ref(db, `rooms/${roomCode}`);
        const now = Date.now();

        await set(roomRef, {
            metadata: {
                host_id: currentUserId,
                created_at: now,
                game_started: false,
                max_players: 8,
                randomize_order: false
            },
            players: {
                [currentUserId]: {
                    name: hostName,
                    character_id: characterId,
                    color_index: colorIndex,
                    is_ready: true,
                    is_connected: true,
                    is_host: true,
                    joined_at: now
                }
            }
        });

        console.log('Room created:', roomCode);
        return true;
    } catch (error) {
        console.error('Create room error:', error);
        window.firebaseError = error.message;
        return false;
    }
};

// Check if a room exists
window.roomExists = async function(roomCode) {
    try {
        const roomRef = ref(db, `rooms/${roomCode}/metadata`);
        const snapshot = await get(roomRef);
        return snapshot.exists();
    } catch (error) {
        console.error('Room check error:', error);
        return false;
    }
};

// Join an existing room
window.joinRoom = async function(roomCode, playerName, characterId, colorIndex) {
    if (!currentUserId) {
        console.error('Not authenticated');
        return false;
    }

    try {
        // Check if room exists
        const roomRef = ref(db, `rooms/${roomCode}/metadata`);
        const snapshot = await get(roomRef);

        if (!snapshot.exists()) {
            window.firebaseError = 'Room not found';
            return false;
        }

        const metadata = snapshot.val();
        if (metadata.game_started) {
            window.firebaseError = 'Game already started';
            return false;
        }

        // Add player to room
        const playerRef = ref(db, `rooms/${roomCode}/players/${currentUserId}`);
        await set(playerRef, {
            name: playerName,
            character_id: characterId,
            color_index: colorIndex,
            is_ready: false,
            is_connected: true,
            is_host: false,
            joined_at: Date.now()
        });

        console.log('Joined room:', roomCode);
        return true;
    } catch (error) {
        console.error('Join room error:', error);
        window.firebaseError = error.message;
        return false;
    }
};

// Leave a room
window.leaveRoom = async function(roomCode) {
    if (!currentUserId) return;

    try {
        const playerRef = ref(db, `rooms/${roomCode}/players/${currentUserId}`);
        await remove(playerRef);
        console.log('Left room:', roomCode);
    } catch (error) {
        console.error('Leave room error:', error);
    }
};

// Update player ready status
window.setPlayerReady = async function(roomCode, isReady) {
    if (!currentUserId) return false;

    try {
        const playerRef = ref(db, `rooms/${roomCode}/players/${currentUserId}/is_ready`);
        await set(playerRef, isReady);
        return true;
    } catch (error) {
        console.error('Set ready error:', error);
        return false;
    }
};

// Update player appearance (character, color, and optionally name)
window.updatePlayerAppearance = async function(roomCode, characterId, colorIndex, name = null) {
    if (!currentUserId) return false;

    try {
        const playerRef = ref(db, `rooms/${roomCode}/players/${currentUserId}`);
        const updateData = {
            character_id: characterId,
            color_index: colorIndex
        };
        if (name !== null) {
            updateData.name = name;
        }
        await update(playerRef, updateData);
        console.log('Updated appearance:', characterId, colorIndex, name);
        return true;
    } catch (error) {
        console.error('Update appearance error:', error);
        return false;
    }
};

// Start the game (host only)
window.startGame = async function(roomCode) {
    if (!currentUserId) return false;

    try {
        const metadataRef = ref(db, `rooms/${roomCode}/metadata/game_started`);
        await set(metadataRef, true);
        console.log('Game started');
        return true;
    } catch (error) {
        console.error('Start game error:', error);
        return false;
    }
};

// Write game state (host only)
window.writeGameState = async function(roomCode, stateJson) {
    try {
        const stateRef = ref(db, `rooms/${roomCode}/game_state`);
        const state = JSON.parse(stateJson);
        await set(stateRef, state);
        return true;
    } catch (error) {
        console.error('Write game state error:', error);
        return false;
    }
};

// Submit an action (clients)
window.submitAction = async function(roomCode, actionJson) {
    if (!currentUserId) return false;

    try {
        const actionsRef = ref(db, `rooms/${roomCode}/actions`);
        const action = JSON.parse(actionJson);
        action.player_id = currentUserId;
        action.timestamp = Date.now();
        action.processed = false;

        await push(actionsRef, action);
        return true;
    } catch (error) {
        console.error('Submit action error:', error);
        return false;
    }
};

// Mark an action as processed (host)
window.markActionProcessed = async function(roomCode, actionId) {
    try {
        const actionRef = ref(db, `rooms/${roomCode}/actions/${actionId}/processed`);
        await set(actionRef, true);
        return true;
    } catch (error) {
        console.error('Mark action error:', error);
        return false;
    }
};

// Subscribe to game state changes
window.subscribeToGameState = function(roomCode) {
    const stateRef = ref(db, `rooms/${roomCode}/game_state`);

    // Remove existing listener if any
    if (activeListeners.has('game_state')) {
        off(activeListeners.get('game_state'));
    }

    const unsubscribe = onValue(stateRef, (snapshot) => {
        if (snapshot.exists()) {
            const data = JSON.stringify(snapshot.val());
            window.firebaseGameStateQueue.push(data);
        }
    }, (error) => {
        console.error('Game state subscription error:', error);
        window.firebaseError = error.message;
    });

    activeListeners.set('game_state', stateRef);
};

// Subscribe to actions (host only)
window.subscribeToActions = function(roomCode) {
    const actionsRef = ref(db, `rooms/${roomCode}/actions`);

    if (activeListeners.has('actions')) {
        off(activeListeners.get('actions'));
    }

    const unsubscribe = onValue(actionsRef, (snapshot) => {
        if (snapshot.exists()) {
            const actions = [];
            snapshot.forEach((child) => {
                const action = child.val();
                if (!action.processed) {
                    actions.push({
                        id: child.key,
                        ...action
                    });
                }
            });
            if (actions.length > 0) {
                window.firebaseActionsQueue.push(JSON.stringify(actions));
            }
        }
    }, (error) => {
        console.error('Actions subscription error:', error);
    });

    activeListeners.set('actions', actionsRef);
};

// Subscribe to players list
window.subscribeToPlayers = function(roomCode) {
    console.log('Subscribing to players for room:', roomCode);
    const playersRef = ref(db, `rooms/${roomCode}/players`);

    if (activeListeners.has('players')) {
        console.log('Removing existing players listener');
        off(activeListeners.get('players'));
    }

    const unsubscribe = onValue(playersRef, (snapshot) => {
        console.log('Players snapshot received, exists:', snapshot.exists());
        if (snapshot.exists()) {
            const players = [];
            snapshot.forEach((child) => {
                console.log('Player found:', child.key, child.val());
                players.push({
                    id: child.key,
                    ...child.val()
                });
            });
            console.log('Pushing players to queue:', players.length, 'players');
            window.firebasePlayersQueue.push(JSON.stringify(players));
        }
    }, (error) => {
        console.error('Players subscription error:', error);
    });

    activeListeners.set('players', playersRef);
};

// Subscribe to room metadata (for game_started flag)
window.subscribeToMetadata = function(roomCode) {
    const metadataRef = ref(db, `rooms/${roomCode}/metadata`);

    if (activeListeners.has('metadata')) {
        off(activeListeners.get('metadata'));
    }

    const unsubscribe = onValue(metadataRef, (snapshot) => {
        if (snapshot.exists()) {
            window.firebaseMetadata = snapshot.val();
        }
    });

    activeListeners.set('metadata', metadataRef);
};

// Poll for game state updates (called from Rust)
window.pollGameState = function() {
    if (window.firebaseGameStateQueue.length > 0) {
        return window.firebaseGameStateQueue.shift();
    }
    return null;
};

// Poll for action updates (called from Rust)
window.pollActions = function() {
    if (window.firebaseActionsQueue.length > 0) {
        return window.firebaseActionsQueue.shift();
    }
    return null;
};

// Poll for player updates (called from Rust)
window.pollPlayers = function() {
    if (window.firebasePlayersQueue.length > 0) {
        return window.firebasePlayersQueue.shift();
    }
    return null;
};

// Get last Firebase error
window.getFirebaseError = function() {
    const error = window.firebaseError;
    window.firebaseError = null;
    return error;
};

// Check if game has started
window.hasGameStarted = function() {
    return window.firebaseMetadata?.game_started ?? false;
};

// Get randomize order setting
window.getRandomizeOrder = function() {
    return window.firebaseMetadata?.randomize_order ?? false;
};

// Set randomize order (host only)
window.setRandomizeOrder = async function(roomCode, randomize) {
    try {
        const metadataRef = ref(db, `rooms/${roomCode}/metadata/randomize_order`);
        await set(metadataRef, randomize);
        console.log('Set randomize order:', randomize);
        return true;
    } catch (error) {
        console.error('Set randomize order error:', error);
        return false;
    }
};

// Unsubscribe from all listeners
window.unsubscribeAll = function() {
    for (const [key, refValue] of activeListeners) {
        off(refValue);
    }
    activeListeners.clear();
    window.firebaseGameStateQueue = [];
    window.firebaseActionsQueue = [];
    window.firebasePlayersQueue = [];
};

// Delete a room (host cleanup)
window.deleteRoom = async function(roomCode) {
    try {
        const roomRef = ref(db, `rooms/${roomCode}`);
        await remove(roomRef);
        return true;
    } catch (error) {
        console.error('Delete room error:', error);
        return false;
    }
};

console.log('Firebase bridge loaded');
