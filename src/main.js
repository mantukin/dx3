const { invoke } = window.__TAURI__.tauri;
const { listen } = window.__TAURI__.event;
const { open } = window.__TAURI__.shell;
const appWindow = window.__TAURI__.window.getCurrent();

// --- State & Assets ---
let currentState = null;
const canvas = document.getElementById('gamepad-canvas');
const ctx = canvas.getContext('2d');
let bgImage = new Image();
let stickImage = new Image();
let ledImage = new Image();

async function loadAsset(name, targetImage) {
    try {
        const bytes = await invoke('get_image_asset', { name });
        const blob = new Blob([new Uint8Array(bytes)], { type: 'image/webp' });
        const url = URL.createObjectURL(blob);
        targetImage.src = url;
    } catch (e) {
        console.error("Failed to load asset:", name, e);
    }
}

// Load assets and hide loader when done
const assetsPromises = [
    loadAsset('dualsense.webp', bgImage),
    loadAsset('analog_stick_head.webp', stickImage),
    loadAsset('LED.webp', ledImage)
];

const githubImg = new Image();
assetsPromises.push(loadAsset('github_icon.webp', githubImg).then(() => {
    const el = document.getElementById('github-img');
    if (el) el.src = githubImg.src;
}));

Promise.all(assetsPromises).then(() => {
    console.log("Assets loaded");
    
    // Small delay to ensure layout is stable and WebView has painted the background
    setTimeout(() => {
        // Show window now that we are ready (prevents white flash)
        appWindow.show();

        const loader = document.getElementById('app-loader');
        if(loader) {
            loader.style.opacity = '0';
            loader.style.transition = 'opacity 0.3s ease';
            setTimeout(() => loader.remove(), 300);
        }
        // Force initial draw
        window._forceRedraw = true;
    }, 150);
});

// --- UI Elements Cache ---
const el = (id) => document.getElementById(id);

const ui = {
    status: el('status-val'),
    device: el('device-val'),
    battery: el('battery-val'),
    connLine: el('conn-line'),
    connMode: el('conn-mode'),
    connWarning: el('conn-warning'),
    statusVigem: el('status-vigem'),
    statusHidHide: el('status-hidhide'),
    statusXbox: el('status-xbox'),
    btnDlVigem: el('btn-dl-vigem'),
    btnDlHidHide: el('btn-dl-hidhide'),
    btnCheckVigem: el('btn-check-vigem'),
    btnCheckHidHide: el('btn-check-hidhide'),
    chkHide: el('chk-hide'),
    chkMin: el('chk-minimized'),
    btnDebug: el('btn-debug-toggle'),
    debugPanel: el('debug-panel'),
    btnRemap: el('btn-remapping-toggle'),
    remapPanel: el('remapping-panel'),
    mappingList: el('mapping-list'),
    btnReset: el('btn-reset-mappings'),
    btnDisconnect: el('btn-disconnect'),
    // Profiles
    mainProfileCtrl: el('main-profile-ctrl'),
    selProfiles: el('sel-profiles'),
    selProfilesMain: el('sel-profiles-main'),
    btnDeleteProfile: el('btn-delete-profile'),
    btnSaveProfile: el('btn-save-profile'),
    inpProfileName: el('inp-profile-name'),
    hexGrid: el('hex-grid'),
    reportMode: el('report-mode'),
    // Action Picker
    picker: el('action-picker'),
    pickerTitle: el('picker-title'),
    mainOptions: el('main-options'),
    optRecord: el('opt-record'),
    xboxOptions: el('xbox-options'),
    xboxGrid: el('xbox-grid'),
    recordingOverlay: el('recording-overlay'),
    // Deadzones
    dzLeft: el('dz-left-ctrl'),
    dzRight: el('dz-right-ctrl'),
    sldDzLeft: el('sld-dz-left'),
    sldDzRight: el('sld-dz-right'),
    inpDzLeft: el('inp-dz-left'),
    inpDzRight: el('inp-dz-right'),
    boxSensLeft: el('sens-left-box'),
    boxSensRight: el('sens-right-box'),
    sldSensLeft: el('sld-sens-left'),
    sldSensRight: el('sld-sens-right'),
    inpSensLeft: el('inp-sens-left'),
    inpSensRight: el('inp-sens-right'),
    // Touchpad Sens
    sensTouchCtrl: el('sens-touch-ctrl'),
    sldSensTouch: el('sld-sens-touch'),
    inpSensTouch: el('inp-sens-touch'),
    // Main RGB
    rgbCtrl: el('rgb-ctrl'),
    mainSldR: el('main-sld-r'),
    mainSldG: el('main-sld-g'),
    mainSldB: el('main-sld-b'),
    mainSldBright: el('main-sld-bright'),
    selPledBright: el('sel-pled-bright'),
    mainColorPreview: el('main-color-preview'),
    chkBatLed: el('chk-bat-led'),
    // Adaptive Triggers
    selTriggerL2Mode: el('sel-trigger-l2-mode'),
    sldTriggerL2Start: el('sld-trigger-l2-start'),
    sldTriggerL2Force: el('sld-trigger-l2-force'),
    selTriggerR2Mode: el('sel-trigger-r2-mode'),
    sldTriggerR2Start: el('sld-trigger-r2-start'),
    sldTriggerR2Force: el('sld-trigger-r2-force'),
    triggerL2Ctrl: el('trigger-l2-ctrl'),
    triggerR2Ctrl: el('trigger-r2-ctrl'),
    // Fuzzer
    btnFuzzer: el('btn-fuzzer'),
    fuzzerStatus: el('fuzzer-status'),
    devicePath: el('device-path'),
    lastWrite: el('last-write-status'),
    packetHex: el('packet-hex'),
    // Manual
    inpRepId: el('inp-rep-id'),
    inpFlagOff: el('inp-flag-off'),
    inpRgbOff: el('inp-rgb-off'),
    inpPled: el('inp-pled'),
    inpPledBright: el('inp-pled-bright'),
    inpPledBrightOff: el('inp-pled-bright-off'),
    inpBtFlags: el('inp-bt-flags'),
    inpBtFlags2: el('inp-bt-flags2'),
    inpBtLen: el('inp-bt-len'),
    chkFeature: el('chk-feature'),
    sldR: el('sld-r'),
    sldG: el('sld-g'),
    sldB: el('sld-b'),
    btnManual: el('btn-manual-send'),
    // Config
    chkNoPeriod: el('chk-no-periodic'),
    selCrc: el('sel-crc'),
    // Scanner
    btnSweep: el('btn-sweep'),
    inpSweepSpeed: el('inp-sweep-speed'),
    // Pinpoint
    inpPpOff: el('inp-pp-off'),
    inpPpVal: el('inp-pp-val'),
    btnPp: el('btn-pp-send'),
    // Logs
    logDevices: el('log-devices'),
    btnProto: el('btn-proto-scan'),
    logProto: el('log-proto'),
    githubIcon: el('github-icon')
};

// --- Initialization ---

// Check build mode (Dev vs Release)
invoke('is_dev').then(isDev => {
    if (!isDev) {
        // In Release mode, remove debug controls entirely
        if (ui.btnDebug) ui.btnDebug.style.display = 'none';
        if (ui.debugPanel) ui.debugPanel.style.display = 'none';
        // Also remove Fuzzer button as it's a dev tool
        if (ui.btnFuzzer) ui.btnFuzzer.style.display = 'none';
        // Remove Protocol Scan button
        if (ui.btnProto) ui.btnProto.style.display = 'none';
    }
});

// Open GitHub link in browser
ui.githubIcon.addEventListener('click', (e) => {
    e.preventDefault();
    open('https://github.com/mantukin/dx3');
});

ui.btnDlVigem.addEventListener('click', () => {
    open('https://github.com/nefarius/ViGEmBus/releases/latest');
});

ui.btnDlHidHide.addEventListener('click', () => {
    open('https://github.com/nefarius/HidHide/releases/latest');
});

// Driver Check Flags
let ignoreVigemUpdate = false;
let ignoreHidHideUpdate = false;

// Driver Check Logic
const fakeCheck = (el, type) => {
    el.textContent = "Checking...";
    el.className = "value warn"; // Yellow color
    
    // Lock UI updates for this element
    if (type === 'vigem') ignoreVigemUpdate = true;
    if (type === 'hidhide') ignoreHidHideUpdate = true;

    // Trigger backend re-initialization
    invoke('trigger_driver_refresh').catch(e => console.error("Driver refresh failed:", e));

    // Release lock after 1.5s to allow user to see "Checking..."
    setTimeout(() => {
        if (type === 'vigem') ignoreVigemUpdate = false;
        if (type === 'hidhide') ignoreHidHideUpdate = false;
    }, 1500);
};

if(ui.btnCheckVigem) {
    ui.btnCheckVigem.addEventListener('click', () => fakeCheck(ui.statusVigem, 'vigem'));
}

if(ui.btnCheckHidHide) {
    ui.btnCheckHidHide.addEventListener('click', () => fakeCheck(ui.statusHidHide, 'hidhide'));
}

// Gamepad Button Hitboxes (Original coordinates from 1669x1046 image)
const HITBOXES = {
    'Cross': { type: 'circle', x: 1363, y: 313 + 123, r: 60 },
    'Circle': { type: 'circle', x: 1363 + 123, y: 313, r: 60 },
    'Square': { type: 'circle', x: 1363 - 123, y: 313, r: 60 },
    'Triangle': { type: 'circle', x: 1363, y: 313 - 123, r: 60 },
    'DpadUp': { type: 'rect', x: 305, y: 310 - 100, w: 90, h: 90 },
    'DpadDown': { type: 'rect', x: 305, y: 310 + 100, w: 90, h: 90 },
    'DpadLeft': { type: 'rect', x: 305 - 100, y: 310, w: 90, h: 90 },
    'DpadRight': { type: 'rect', x: 305 + 100, y: 310, w: 90, h: 90 },
    'L1': { type: 'rect', colOffset: -220, y: 90, w: 160, h: 20, custom: true },
    'R1': { type: 'rect', colOffset: 220, y: 90, w: 160, h: 20, custom: true },
    'L2': { type: 'rect', colOffset: -220, y: 55, w: 160, h: 24, custom: true },
    'R2': { type: 'rect', colOffset: 220, y: 55, w: 160, h: 24, custom: true },
    'LeftStick': { type: 'circle', x: 565, y: 535, r: 100, isAxis: true },
    'RightStick': { type: 'circle', x: 1105, y: 535, r: 100, isAxis: true },
    'L3': { type: 'circle', x: 565, y: 535, r: 50 },
    'R3': { type: 'circle', x: 1105, y: 535, r: 50 },
    'Share': { type: 'rect', x: 445, y: 130, w: 50, h: 95 },
    'Options': { type: 'rect', x: 1230, y: 130, w: 50, h: 95 },
    'PS': { type: 'circle', x: 835, y: 525, r: 50 },
    'Mute': { type: 'rect', x: 835, y: 625, w: 80, h: 30 },
    'Touchpad': { type: 'rect', x: 835, y: 200, w: 650, h: 310, isAxis: true },
    'TouchpadLeft': { type: 'rect', x: 672, y: 200, w: 280, h: 260 },
    'TouchpadRight': { type: 'rect', x: 998, y: 200, w: 280, h: 260 }
};

let hoveredButton = null;
let selectedButton = null;
let layoutData = { offsetX: 0, offsetY: 0, scale: 1 };
let isAppendingMapping = false; // If true, add to targets. If false, replace targets.

// Helper to get screen coords for a hitbox
function getScreenBox(key) {
    const box = HITBOXES[key];
    const cx = canvas.width / 2;
    const { offsetX, offsetY, scale } = layoutData;

    if (box.custom) {
        // Floating boxes are relative to canvas center
        return {
            type: 'rect',
            x: cx + box.colOffset - box.w / 2,
            y: box.y,
            w: box.w,
            h: box.h
        };
    }

    const p = (x, y) => ({ x: offsetX + x * scale, y: offsetY + y * scale });
    const sz = (v) => v * scale;

    if (box.type === 'circle') {
        const center = p(box.x, box.y);
        return { type: 'circle', x: center.x, y: center.y, r: sz(box.r) };
    } else {
        const center = p(box.x, box.y);
        return { type: 'rect', x: center.x - sz(box.w) / 2, y: center.y - sz(box.h) / 2, w: sz(box.w), h: sz(box.h) };
    }
}

function isPointInBox(px, py, key) {
    const box = getScreenBox(key);
    if (box.type === 'circle') {
        const dist = Math.sqrt((px - box.x) ** 2 + (py - box.y) ** 2);
        return dist <= box.r;
    } else {
        return px >= box.x && px <= box.x + box.w && py >= box.y && py <= box.y + box.h;
    }
}

canvas.addEventListener('mousemove', (e) => {
    const rect = canvas.getBoundingClientRect();
    const mx = e.clientX - rect.left;
    const my = e.clientY - rect.top;

    let found = null;
    // Iterate in reverse to catch axes (Sticks) which overlap with L3/R3
    const keys = Object.keys(HITBOXES);
    for (let i = keys.length - 1; i >= 0; i--) {
        if (isPointInBox(mx, my, keys[i])) {
            found = keys[i];
            break;
        }
    }

    if (found !== hoveredButton) {
        hoveredButton = found;
        canvas.style.cursor = found ? 'pointer' : 'default';
        window._forceRedraw = true;
        hasNewState = true;
    }
});

window.addEventListener('mousedown', (e) => {
    if (e.target === canvas && hoveredButton) {
        selectedButton = hoveredButton;
        isAppendingMapping = false; 
        showPicker(e.clientX, e.clientY);
        window._forceRedraw = true;
        hasNewState = true;
        return;
    }

    if (ui.picker.style.display === 'flex') {
        if (!ui.picker.contains(e.target)) {
            ui.picker.style.display = 'none';
            selectedButton = null;
            window._forceRedraw = true;
            hasNewState = true;
        }
    }
});

function showPicker(x, y) {
    ui.picker.style.display = 'flex';
    ui.mainOptions.style.display = 'flex';
    ui.xboxOptions.style.display = 'none';

    const margin = 15;
    const pickerWidth = ui.picker.offsetWidth || 200;
    const pickerHeight = ui.picker.offsetHeight || 300;

    let posX = x + 20;
    let posY = y + 20;

    if (posX + pickerWidth > window.innerWidth - margin) {
        posX = x - pickerWidth - 10;
    }
    if (posY + pickerHeight > window.innerHeight - margin) {
        posY = window.innerHeight - pickerHeight - margin;
    }

    ui.picker.style.left = `${Math.max(margin, posX)}px`;
    ui.picker.style.top = `${Math.max(margin, posY)}px`;

    ui.pickerTitle.textContent = `Map: ${selectedButton}`;

    const isStick = (selectedButton === 'LeftStick' || selectedButton === 'RightStick');
    const isTouchpadWhole = (selectedButton === 'Touchpad');
    const isAxis = HITBOXES[selectedButton].isAxis || isStick;

    // Hide recording for sticks and Whole Touchpad
    ui.optRecord.style.display = (isStick || isTouchpadWhole) ? 'none' : 'block';
    
    // Xbox Option
    const optXbox = el('opt-xbox');
    if (isStick) {
        optXbox.textContent = 'Assign Xbox Stick';
        optXbox.style.display = 'block';
    } else if (isTouchpadWhole) {
        optXbox.style.display = 'none'; // Whole Touchpad = Mouse Only
    } else {
        optXbox.textContent = 'Assign Xbox Action';
        optXbox.style.display = 'block';
    }

    // Show/Hide Axis specific options
    el('opt-mouse-move').style.display = isAxis ? 'block' : 'none';
    el('opt-mouse-scroll').style.display = isAxis ? 'block' : 'none';
}

function getMappingFor(source) {
    let m = currentState.mappings.find(x => x.source === source);
    if (!m) {
        // Create if missing (safety)
        m = { source: source, targets: [] };
        currentState.mappings.push(m);
    }
    return m;
}

el('opt-xbox').onclick = () => {
    ui.mainOptions.style.display = 'none';
    ui.xboxOptions.style.display = 'flex';
    ui.pickerTitle.textContent = `Select Xbox Target`;

    ui.xboxGrid.innerHTML = '';
    const isStick = (selectedButton === 'LeftStick' || selectedButton === 'RightStick');

    if (!isStick) {
        Object.entries(XBOX_NAMES).forEach(([bit, name]) => {
            const btn = document.createElement('div');
            btn.className = 'btn-xbox-pick';
            btn.textContent = name;
            btn.onclick = () => pickXboxTarget(selectedButton, parseInt(bit));
            ui.xboxGrid.appendChild(btn);
        });
    }

    const targets = isStick ? ['LS', 'RS'] : ['LT', 'RT'];
    targets.forEach(t => {
        const btn = document.createElement('div');
        btn.className = 'btn-xbox-pick';
        btn.textContent = t;
        btn.onclick = () => pickXboxTriggerTarget(selectedButton, 'Xbox' + t);
        ui.xboxGrid.appendChild(btn);
    });
};

el('opt-back').onclick = () => {
    showPicker(parseInt(ui.picker.style.left), parseInt(ui.picker.style.top));
};

ui.optRecord.onclick = () => {
    recordingFor = selectedButton;
    ui.picker.style.display = 'none';
    ui.recordingOverlay.style.display = 'block';
};

el('opt-mouse-move').onclick = async () => {
    const m = getMappingFor(selectedButton);
    m.targets = [{ MouseMove: { x_speed: 25.0, y_speed: 25.0 } }];
    await invoke('update_mappings', { mappings: currentState.mappings });
    ui.picker.style.display = 'none';
    selectedButton = null;
    renderMappings();
};

el('opt-mouse-scroll').onclick = async () => {
    const m = getMappingFor(selectedButton);
    m.targets = [{ MouseScroll: { speed: 1.0 } }];
    await invoke('update_mappings', { mappings: currentState.mappings });
    ui.picker.style.display = 'none';
    selectedButton = null;
    renderMappings();
};

el('opt-clear').textContent = 'Reset to Default';
el('opt-clear').onclick = async () => {
    const m = getMappingFor(selectedButton);
    const def = DEFAULT_MAPPINGS[selectedButton];

    let defTargets = [];
    if (Array.isArray(def)) {
        defTargets = def;
    } else if (def) {
        defTargets = [def];
    }

    m.targets = JSON.parse(JSON.stringify(defTargets));

    await invoke('update_mappings', { mappings: currentState.mappings });
    ui.picker.style.display = 'none';
    selectedButton = null;
    renderMappings();
};

// Generate Hex Grid
for (let i = 0; i < 80; i++) {
    const d = document.createElement('div');
    d.className = 'hex-cell';
    d.id = 'hex-' + i;
    d.innerHTML = `00<span>${i}</span>`;
    ui.hexGrid.appendChild(d);
}

// --- Event Listeners ---

// Top Bar
function updatePanelButtonTexts() {
    const debugVisible = ui.debugPanel.style.display === 'flex';
    const remapVisible = ui.remapPanel.style.display === 'flex';
    
    ui.btnDebug.textContent = debugVisible ? 'Hide Debug' : 'Show Debug';
    ui.btnRemap.textContent = remapVisible ? 'Hide Remapping' : 'Remapping';
}

ui.btnDebug.addEventListener('click', () => {
    ui.picker.style.display = 'none';
    ui.remapPanel.style.display = 'none';
    invoke('toggle_debug');
    ui.debugPanel.style.display = (ui.debugPanel.style.display === 'flex') ? 'none' : 'flex';
    updatePanelButtonTexts();
});

ui.btnRemap.addEventListener('click', () => {
    ui.picker.style.display = 'none';
    ui.debugPanel.style.display = 'none';
    const isVisible = ui.remapPanel.style.display === 'flex';
    ui.remapPanel.style.display = isVisible ? 'none' : 'flex';
    if (!isVisible) {
        renderMappings();
        refreshProfilesList();
    }
    updatePanelButtonTexts();
});

async function refreshProfilesList() {
    const profiles = await invoke('get_profiles');
    const updateSelect = (sel) => {
        sel.innerHTML = '<option value="Default">Default</option>';
        profiles.forEach(p => {
            if (p === 'Default') return;
            const opt = document.createElement('option');
            opt.value = p;
            opt.textContent = p;
            if (currentState && currentState.current_profile_name === p) {
                opt.selected = true;
            }
            sel.appendChild(opt);
        });
        // Ensure Default is selected if active
        if (currentState && currentState.current_profile_name === 'Default') {
            sel.value = 'Default';
        }
    };

    updateSelect(ui.selProfiles);
    updateSelect(ui.selProfilesMain);
}

ui.btnSaveProfile.addEventListener('click', async () => {
    const name = ui.inpProfileName.value.trim();
    if (!name) {
        alert('Please enter a profile name.');
        return;
    }
    await invoke('save_profile', { name: name });
    ui.inpProfileName.value = '';
    // After saving, immediately load it to make it active
    await invoke('load_profile', { name: name });
    refreshProfilesList();
});

const handleProfileChange = async (e) => {
    const name = e.target.value;
    if (!name) return;
    await invoke('load_profile', { name: name });
    
    // Fetch updated state to sync UI (sliders, colors, deadzones)
    const json = await invoke('get_initial_state');
    currentState = JSON.parse(json);
    syncUiToState(currentState);
    
    renderMappings();
};

ui.selProfiles.addEventListener('change', handleProfileChange);
ui.selProfilesMain.addEventListener('change', handleProfileChange);

ui.btnDeleteProfile.addEventListener('click', async () => {
    const name = ui.selProfiles.value;
    if (!name) return;
    if (confirm(`Delete profile "${name}"?`)) {
        await invoke('delete_profile', { name: name });
        refreshProfilesList();
    }
});

ui.btnReset.addEventListener('click', async () => {
    if (confirm('Reset all button mappings to default Xbox layout? This will also switch to the Default profile.')) {
        await invoke('reset_mappings');
        // Refresh state
        invoke('get_initial_state').then(json => {
            currentState = JSON.parse(json);
            syncUiToState(currentState); // Sync UI elements
            renderMappings();
            refreshProfilesList();
        });
    }
});

let recordingFor = null; // { source: string }

const XBOX_NAMES = {
    0x1000: 'A', 0x2000: 'B', 0x4000: 'X', 0x8000: 'Y',
    0x0100: 'LB', 0x0200: 'RB', 0x0040: 'L3', 0x0080: 'R3',
    0x0010: 'Start', 0x0020: 'Back', 0x0400: 'Guide',
    0x0001: 'Up', 0x0002: 'Down', 0x0004: 'Left', 0x0008: 'Right'
};

const PHYSICAL_TO_XBOX = {
    'Cross': 0x1000, 'Circle': 0x2000, 'Square': 0x4000, 'Triangle': 0x8000,
    'L1': 0x0100, 'R1': 0x0200, 'L3': 0x0040, 'R3': 0x0080,
    'Options': 0x0010, 'Share': 0x0020, 'PS': 0x0400,
    'DpadUp': 0x0001, 'DpadDown': 0x0002, 'DpadLeft': 0x0004, 'DpadRight': 0x0008,
    'L2': null, 'R2': null, 'Mute': null, 'Touchpad': null, 'TouchpadLeft': null, 'TouchpadRight': null
};

const DEFAULT_MAPPINGS = {
    'Cross': { Xbox: 0x1000 }, 'Circle': { Xbox: 0x2000 }, 'Square': { Xbox: 0x4000 }, 'Triangle': { Xbox: 0x8000 },
    'L1': { Xbox: 0x0100 }, 'R1': { Xbox: 0x0200 }, 'L3': { Xbox: 0x0040 }, 'R3': { Xbox: 0x0080 },
    'Options': { Xbox: 0x0010 }, 'Share': { Xbox: 0x0020 }, 'PS': { Xbox: 0x0400 },
    'Mute': [], 'Touchpad': [], 'TouchpadLeft': [], 'TouchpadRight': [],
    'DpadUp': { Xbox: 0x0001 }, 'DpadDown': { Xbox: 0x0002 }, 'DpadLeft': { Xbox: 0x0004 }, 'DpadRight': { Xbox: 0x0008 },
    'LeftStick': 'XboxLS', 'RightStick': 'XboxRS', 'L2': 'XboxLT', 'R2': 'XboxRT'
};

function isMappingModified(m) {
    const def = DEFAULT_MAPPINGS[m.source];

    let defTargets = [];
    if (Array.isArray(def)) {
        defTargets = def;
    } else if (def) {
        defTargets = [def];
    }

    return JSON.stringify(m.targets) !== JSON.stringify(defTargets);
}

function getKeyName(vk) {
    // Basic mapping for common keys
    if (vk >= 65 && vk <= 90) return String.fromCharCode(vk); // A-Z
    if (vk >= 48 && vk <= 57) return String.fromCharCode(vk); // 0-9

    const special = {
        8: 'Backspace', 9: 'Tab', 13: 'Enter', 16: 'Shift', 17: 'Ctrl', 18: 'Alt',
        20: 'Caps', 27: 'Esc', 32: 'Space', 33: 'PgUp', 34: 'PgDn', 35: 'End', 36: 'Home',
        37: 'Left', 38: 'Up', 39: 'Right', 40: 'Down', 45: 'Ins', 46: 'Del',
        91: 'Win', 93: 'Menu', 144: 'NumLock'
    };
    if (vk >= 112 && vk <= 123) return 'F' + (vk - 111); // F1-F12

    return special[vk] || `Key ${vk}`;
}

// Build the mapping list DOM (call only when data changes)
function renderMappings() {
    if (!currentState || !currentState.mappings) return;

    ui.mappingList.innerHTML = '';

    currentState.mappings.forEach(m => {
        const container = document.createElement('div');
        container.style.display = 'flex';
        container.style.flexDirection = 'column';
        container.style.gap = '2px';

        const row = document.createElement('div');
        row.className = 'mapping-row';
        row.id = `map-row-${m.source}`;

        const source = document.createElement('div');
        source.className = 'mapping-source';
        source.textContent = m.source;

        const targets = document.createElement('div');
        targets.className = 'mapping-targets';

        m.targets.forEach((t, idx) => {
            const tag = document.createElement('div');
            let type = '';
            let label = '';

            if (t.Xbox !== undefined) {
                type = 'xbox';
                label = `Xbox ${XBOX_NAMES[t.Xbox] || t.Xbox}`;
            } else if (t === 'XboxLT') {
                type = 'xbox';
                label = 'Xbox LT';
            } else if (t === 'XboxRT') {
                type = 'xbox';
                label = 'Xbox RT';
            } else if (t === 'XboxLS') {
                type = 'xbox';
                label = 'Xbox LS';
            } else if (t === 'XboxRS') {
                type = 'xbox';
                label = 'Xbox RS';
            } else if (t.Keyboard !== undefined) {
                type = 'kb';
                label = getKeyName(t.Keyboard);
            } else if (t.Mouse !== undefined) {
                type = 'mouse';
                label = `Mouse ${['Left', 'Middle', 'Right'][t.Mouse] || t.Mouse}`;
            } else if (t.MouseMove !== undefined) {
                type = 'mouse';
                label = 'Mouse Move';
            } else if (t.MouseScroll !== undefined) {
                type = 'mouse';
                label = 'Mouse Scroll';
            }

            tag.className = `target-tag ${type}`;
            tag.innerHTML = `${label} <span class="remove" data-source="${m.source}" data-idx="${idx}">&times;</span>`;

            tag.querySelector('.remove').addEventListener('click', (e) => {
                e.stopPropagation();
                removeTarget(m.source, idx);
            });

            targets.appendChild(tag);
        });

        const btnAdd = document.createElement('button');
        btnAdd.className = 'btn-add-target';
        btnAdd.textContent = (recordingFor === m.source) ? 'CANCEL RECORD' : '+ Add';
        btnAdd.onclick = () => {
            if (recordingFor === m.source) {
                recordingFor = null;
                isAppendingMapping = false;
                renderMappings();
            } else {
                isAppendingMapping = true; // Table button means append
                startRecording(m.source);
            }
        };

        row.appendChild(source);
        row.appendChild(targets);
        row.appendChild(btnAdd);
        container.appendChild(row);

        // Show Xbox Picker if recording this source
        if (recordingFor === m.source) {
            const picker = document.createElement('div');
            picker.className = 'xbox-selector';

            const isStick = (m.source === 'LeftStick' || m.source === 'RightStick');
            const isTouchpad = (m.source === 'Touchpad');

            // Hide standard Xbox buttons for Sticks AND Touchpad
            if (!isStick && !isTouchpad) {
                Object.entries(XBOX_NAMES).forEach(([bit, name]) => {
                    const btn = document.createElement('div');
                    btn.className = 'btn-xbox-pick';
                    btn.textContent = name;
                    btn.onclick = (e) => {
                        e.stopPropagation();
                        pickXboxTarget(m.source, parseInt(bit));
                    };
                    picker.appendChild(btn);
                });
            }

            // Add Triggers (LT/RT) only for normal buttons (Not Stick, Not Touchpad)
            // Add Stick clicks (LS/RS) only for Sticks
            if (!isTouchpad) {
                const targets = isStick ? ['LS', 'RS'] : ['LT', 'RT'];
                targets.forEach(t => {
                    const btn = document.createElement('div');
                    btn.className = 'btn-xbox-pick';
                    btn.textContent = t;
                    btn.onclick = (e) => {
                        e.stopPropagation();
                        pickXboxTriggerTarget(m.source, 'Xbox' + t);
                    };
                    picker.appendChild(btn);
                });
            }

            // Add Mouse Options for Sticks AND Touchpad
            if (isStick || isTouchpad) {
                const mouseMoveBtn = document.createElement('div');
                mouseMoveBtn.className = 'btn-xbox-pick';
                mouseMoveBtn.style.background = '#5c6370';
                mouseMoveBtn.textContent = 'Mouse Move';
                mouseMoveBtn.onclick = (e) => {
                    e.stopPropagation();
                    selectedButton = m.source;
                    isAppendingMapping = true;
                    el('opt-mouse-move').click();
                };
                picker.appendChild(mouseMoveBtn);

                const mouseScrollBtn = document.createElement('div');
                mouseScrollBtn.className = 'btn-xbox-pick';
                mouseScrollBtn.style.background = '#5c6370';
                mouseScrollBtn.textContent = 'Mouse Scroll';
                mouseScrollBtn.onclick = (e) => {
                    e.stopPropagation();
                    selectedButton = m.source;
                    isAppendingMapping = true;
                    el('opt-mouse-scroll').click();
                };
                picker.appendChild(mouseScrollBtn);
            }

            container.appendChild(picker);
        }

        ui.mappingList.appendChild(container);
    });

    updateMappingsActiveState();
}

function pickXboxTarget(source, bit) {
    const m = getMappingFor(source);
    if (m) {
        if (isAppendingMapping) {
            if (!m.targets.some(t => t.Xbox === bit)) {
                m.targets.push({ Xbox: bit });
            }
        } else {
            m.targets = [{ Xbox: bit }];
        }
        invoke('update_mappings', { mappings: currentState.mappings });
    }
    recordingFor = null;
    isAppendingMapping = false;
    renderMappings();
}

function pickXboxTriggerTarget(source, target) {
    const m = getMappingFor(source);
    if (m) {
        if (isAppendingMapping) {
            if (!m.targets.some(t => t === target)) {
                m.targets.push(target);
            }
        } else {
            m.targets = [target];
        }
        invoke('update_mappings', { mappings: currentState.mappings });
    }
    recordingFor = null;
    isAppendingMapping = false;
    renderMappings();
}

// Update only the .active class without rebuilding DOM
function updateMappingsActiveState() {
    if (!currentState || !currentState.mappings) return;

    // Synthetic states for Touchpad Halves
    const gp = { ...currentState.gamepad };
    if (gp.btn_touchpad && gp.touch_active) {
        if (gp.touch_x < 960) gp.btn_touchpad_left = true;
        else gp.btn_touchpad_right = true;
    }

    // Gamepad Recording Detection
    if (recordingFor) {
        for (const [phys, bit] of Object.entries(PHYSICAL_TO_XBOX)) {
            const field = getGamepadField(phys);
            const val = gp[field];

            // Check if button is pressed (boolean) or trigger is pulled > 50% (float)
            const isPressed = (typeof val === 'boolean') ? val : (val > 0.5);

            if (isPressed) {
                // Button pressed while recording!
                const m = getMappingFor(recordingFor);
                if (m) {
                    if (bit !== null) {
                        m.targets = [{ Xbox: bit }];
                        invoke('update_mappings', { mappings: currentState.mappings });
                    }
                }
                recordingFor = null;
                selectedButton = null;
                ui.recordingOverlay.style.display = 'none';
                renderMappings();
                return;
            }
        }
    }

    currentState.mappings.forEach(m => {
        const row = document.getElementById(`map-row-${m.source}`);
        if (row) {
            const isPressed = gp[getGamepadField(m.source)];
            if (isPressed) {
                row.classList.add('active');
            } else {
                row.classList.remove('active');
            }
        }
    });
}

function getGamepadField(phys) {
    const map = {
        'Cross': 'btn_cross', 'Circle': 'btn_circle', 'Square': 'btn_square', 'Triangle': 'btn_triangle',
        'L1': 'btn_l1', 'R1': 'btn_r1', 'L3': 'btn_l3', 'R3': 'btn_r3',
        'Options': 'btn_options', 'Share': 'btn_share', 'PS': 'btn_ps', 'Touchpad': 'btn_touchpad',
        'Mute': 'btn_mute', 'TouchpadLeft': 'btn_touchpad_left', 'TouchpadRight': 'btn_touchpad_right',
        'DpadUp': 'dpad_up', 'DpadDown': 'dpad_down', 'DpadLeft': 'dpad_left', 'DpadRight': 'dpad_right'
    };
    return map[phys] || phys;
}

function startRecording(source) {
    recordingFor = source;
    renderMappings();
}

function removeTarget(source, index) {
    const m = currentState.mappings.find(x => x.source === source);
    if (m) {
        m.targets.splice(index, 1);
        invoke('update_mappings', { mappings: currentState.mappings });
        renderMappings();
    }
}

// Make functions available if needed
window.startRecording = startRecording;
window.removeTarget = removeTarget;

// Keyboard Recording
window.addEventListener('keydown', (e) => {
    if (recordingFor) {
        // Block keyboard recording for analog inputs
        if (recordingFor === 'LeftStick' || recordingFor === 'RightStick' || recordingFor === 'Touchpad') {
            return;
        }

        e.preventDefault();
        const vk = e.keyCode;
        const m = getMappingFor(recordingFor);
        if (m) {
            if (isAppendingMapping) {
                if (!m.targets.some(t => t.Keyboard === vk)) {
                    m.targets.push({ Keyboard: vk });
                }
            } else {
                m.targets = [{ Keyboard: vk }];
            }
            invoke('update_mappings', { mappings: currentState.mappings });
        }
        recordingFor = null;
        selectedButton = null;
        isAppendingMapping = false;
        ui.recordingOverlay.style.display = 'none';
        renderMappings();
    }
});

// Mouse Recording (Simple)
window.addEventListener('mousedown', (e) => {
    if (recordingFor) {
        // Block mouse button recording for analog inputs
        if (recordingFor === 'LeftStick' || recordingFor === 'RightStick' || recordingFor === 'Touchpad') {
            // Note: Clicks on the inline picker buttons (Mouse Move/Scroll) are handled 
            // by their own onclick handlers which call stopPropagation(), so they won't reach here.
            return;
        }

        // Ignore clicks on UI elements that are meant to be interactive
        if (e.target.closest('button') ||
            e.target.closest('input') ||
            e.target.closest('select') ||
            e.target.closest('label') ||
            e.target.closest('.btn-xbox-pick') ||
            e.target.closest('.remove') ||
            e.target.closest('#action-picker')) {
            return;
        }

        // Only prevent default if we're actually recording to avoid breaking UI clicks
        e.preventDefault();
        const m = getMappingFor(recordingFor);
        if (m) {
            if (isAppendingMapping) {
                if (!m.targets.some(t => t.Mouse === e.button)) {
                    m.targets.push({ Mouse: e.button });
                }
            } else {
                m.targets = [{ Mouse: e.button }];
            }
            invoke('update_mappings', { mappings: currentState.mappings });
        }
        recordingFor = null;
        selectedButton = null;
        isAppendingMapping = false;
        ui.recordingOverlay.style.display = 'none';
        renderMappings();
    }
});

ui.btnDisconnect.addEventListener('click', () => {
    if (confirm('Reconnect controller? (This will attempt to fix connection issues)')) {
        invoke('disconnect_controller');
    }
});

ui.chkHide.addEventListener('change', (e) => invoke('set_hide_controller', { hide: e.target.checked }));
ui.chkMin.addEventListener('change', (e) => invoke('set_start_minimized', { val: e.target.checked }));

// Fuzzer
ui.btnFuzzer.addEventListener('click', () => {
    const active = ui.btnFuzzer.textContent.includes('STOP');
    invoke('set_fuzzer_active', { val: !active });
});

// Manual Params Update
const updateManual = () => {
    const rid = parseInt(ui.inpRepId.value, 16) || 0;
    const flg = parseInt(ui.inpBtFlags.value, 16) || 0;
    const flg2 = parseInt(ui.inpBtFlags2.value, 16) || 0;

    // Explicitly use snake_case for Rust compatibility
    const params = {
        report_id: rid,
        flag_off: parseInt(ui.inpFlagOff.value) || 0,
        rgb_off: parseInt(ui.inpRgbOff.value) || 0,
        player_led: parseInt(ui.inpPled.value) || 0,
        pled_bright: parseInt(ui.inpPledBright.value) || 0,
        pled_bright_off: parseInt(ui.inpPledBrightOff.value) || 0,
        bt_flags: flg,
        bt_flags2: flg2,
        bt_len: parseInt(ui.inpBtLen.value) || 0,
        as_feature: ui.chkFeature.checked,
        r: parseInt(ui.sldR.value),
        g: parseInt(ui.sldG.value),
        b: parseInt(ui.sldB.value)
    };

    console.log("Updating manual params:", params);
    return invoke('set_manual_params', { params: params }).catch(e => console.error("Error setting params:", e));
};

[ui.inpRepId, ui.inpFlagOff, ui.inpRgbOff, ui.inpPled, ui.inpPledBright, ui.inpPledBrightOff, ui.inpBtFlags, ui.inpBtFlags2, ui.inpBtLen, ui.chkFeature, ui.sldR, ui.sldG, ui.sldB]
    .forEach(el => el.addEventListener('change', updateManual));

[ui.sldR, ui.sldG, ui.sldB].forEach(el => el.addEventListener('input', updateManual));

ui.btnManual.addEventListener('click', async () => {
    console.log("Triggering manual send...");
    await updateManual(); // Wait for params to be synced!
    invoke('trigger_manual_send').catch(e => console.error("Send failed:", e));
});

// Config
ui.chkNoPeriod.addEventListener('change', (e) => invoke('set_disable_periodic', { val: e.target.checked }));
ui.selCrc.addEventListener('change', (e) => invoke('set_crc_seed', { val: parseInt(e.target.value) }));

// Scanner
ui.btnSweep.addEventListener('click', () => {
    const active = ui.btnSweep.textContent.includes('STOP');
    invoke('set_sweep_active', { val: !active });
});
ui.inpSweepSpeed.addEventListener('change', (e) => invoke('set_sweep_speed', { val: parseInt(e.target.value) || 250 }));

// Pinpoint
const updatePp = () => {
    invoke('set_pinpoint_params', {
        offset: parseInt(ui.inpPpOff.value) || 0,
        value: parseInt(ui.inpPpVal.value) || 0
    });
};
ui.inpPpOff.addEventListener('change', updatePp);
ui.inpPpVal.addEventListener('change', updatePp);
ui.btnPp.addEventListener('click', () => invoke('trigger_pinpoint_send'));

// Proto
ui.btnProto.addEventListener('click', () => invoke('trigger_protocol_scan'));


// --- State Handling ---

let hasNewState = false;
let lastRenderedGamepadJSON = "";
let lastRenderedMappingsJSON = "";

listen('update-state', (event) => {
    const newState = event.payload;

    // Check if mappings have changed to refresh the list UI
    const mappingsChanged = currentState && JSON.stringify(newState.mappings) !== JSON.stringify(currentState.mappings);
    const profileChanged = currentState && newState.current_profile_name !== currentState.current_profile_name;

    currentState = newState;
    hasNewState = true;

    if (ui.remapPanel.style.display === 'flex') {
        if (mappingsChanged || profileChanged) {
            renderMappings();
            refreshProfilesList();
        }
        updateMappingsActiveState();
    }
});

function animationLoop() {
    if (hasNewState && currentState) {
        render();
        hasNewState = false;
    }
    requestAnimationFrame(animationLoop);
}

requestAnimationFrame(animationLoop);

function syncUiToState(state) {
    ui.chkHide.checked = state.hide_controller;
    ui.chkMin.checked = state.start_minimized;
    ui.sldDzLeft.value = state.deadzone_left;
    ui.inpDzLeft.value = state.deadzone_left;
    ui.sldDzRight.value = state.deadzone_right;
    ui.inpDzRight.value = state.deadzone_right;
    ui.sldSensLeft.value = state.mouse_sens_left;
    ui.inpSensLeft.value = state.mouse_sens_left;
    ui.sldSensRight.value = state.mouse_sens_right;
    ui.inpSensRight.value = state.mouse_sens_right;
    if (state.mouse_sens_touchpad !== undefined) {
        ui.sldSensTouch.value = state.mouse_sens_touchpad;
        ui.inpSensTouch.value = state.mouse_sens_touchpad;
    }
    ui.mainSldR.value = state.rgb_r;
    ui.mainSldG.value = state.rgb_g;
    ui.mainSldB.value = state.rgb_b;
    ui.mainSldBright.value = state.rgb_brightness;
    ui.chkBatLed.checked = state.show_battery_led;
    if (state.player_led_brightness !== undefined) {
        ui.selPledBright.value = state.player_led_brightness;
    }
    // Adaptive Triggers
    ui.selTriggerL2Mode.value = state.trigger_l2_mode || 0;
    ui.sldTriggerL2Start.value = state.trigger_l2_start || 0;
    ui.sldTriggerL2Force.value = state.trigger_l2_force || 0;
    ui.selTriggerR2Mode.value = state.trigger_r2_mode || 0;
    ui.sldTriggerR2Start.value = state.trigger_r2_start || 0;
    ui.sldTriggerR2Force.value = state.trigger_r2_force || 0;
    
    updateTriggerL2();
    updateTriggerR2();
    updateMainColorPreview();
}

invoke('get_initial_state').then((json) => {
    if (json && json !== "{}") {
        currentState = JSON.parse(json);
        syncUiToState(currentState);
        
        ui.debugPanel.style.display = currentState.debug_active ? 'flex' : 'none';
        updatePanelButtonTexts();
        syncManualInputs(currentState);
        
        hasNewState = true; 
        renderMappings();
        refreshProfilesList();
    }
});

function setText(el, text) {
    if (el.textContent !== text) el.textContent = text;
}

function render() {
    if (!currentState) return;

    // Sync canvas resolution for responsiveness
    if (canvas.width !== canvas.clientWidth || canvas.height !== canvas.clientHeight) {
        // Ensure we don't set 0 dimensions
        if (canvas.clientWidth > 0 && canvas.clientHeight > 0) {
            canvas.width = canvas.clientWidth;
            canvas.height = canvas.clientHeight;
            window._forceRedraw = true;
        }
    }

    const s = currentState.gamepad;

    const isConnected = currentState.device_name !== 'None';
    setText(ui.status, isConnected ? 'Connected' : 'Disconnected');
    ui.status.className = 'value ' + (isConnected ? 'active' : 'error');

    // ViGEmBus
    if (!ignoreVigemUpdate) {
        if (currentState.vigembus_available) {
            setText(ui.statusVigem, "OK");
            ui.statusVigem.className = "value active";
            ui.btnDlVigem.style.display = 'none';
        } else {
            setText(ui.statusVigem, "Not Found");
            ui.statusVigem.className = "value error";
            ui.btnDlVigem.style.display = 'inline-block';
        }
    }

    // HidHide
    if (!ignoreHidHideUpdate) {
        if (currentState.hidhide_available) {
            setText(ui.statusHidHide, "OK");
            ui.statusHidHide.className = "value active";
            ui.btnDlHidHide.style.display = 'none';
        } else {
            setText(ui.statusHidHide, "Not Found");
            ui.statusHidHide.className = "value error";
            ui.btnDlHidHide.style.display = 'inline-block';
        }
    }

    // Xbox Device (Virtual Pad)
    if (currentState.virtual_pad_active) {
        setText(ui.statusXbox, "Visible");
        ui.statusXbox.className = "value active";
    } else {
        setText(ui.statusXbox, "Hidden");
        ui.statusXbox.className = "value"; // Neutral color for hidden
    }

    setText(ui.device, currentState.device_name);

    const isPaused = currentState.is_paused;

    const discDisplay = (isConnected || isPaused) ? 'inline-block' : 'none';
    if (ui.btnDisconnect.style.display !== discDisplay) {
        ui.btnDisconnect.style.display = discDisplay;
        ui.btnDisconnect.textContent = 'Reconnect';
        ui.btnDisconnect.style.background = '#722f37';
        ui.btnDisconnect.style.borderColor = '#a33';
    }

    // Connection Mode Handling
    if (currentState.connection_mode && currentState.connection_mode !== "") {
        setText(ui.connMode, currentState.connection_mode);
        const isSimple = currentState.connection_mode.includes('Simple');
        ui.connMode.className = 'value ' + (isSimple ? 'warn' : 'active');
        
        const warnDisplay = isSimple ? 'inline' : 'none';
        if (ui.connWarning.style.display !== warnDisplay) ui.connWarning.style.display = warnDisplay;
    } else {
        setText(ui.connMode, "None");
        ui.connMode.className = "value";
        ui.connWarning.style.display = 'none';
    }

    if (s.battery > 0 || s.is_charging) {
        const batText = `🔋 ${s.battery}%${s.is_charging ? '⚡' : ''}`;
        setText(ui.battery, batText);
        const batClass = 'value ' + (s.battery <= 20 ? 'warn' : 'active');
        if (ui.battery.className !== batClass) ui.battery.className = batClass;
    } else {
        setText(ui.battery, '');
    }

    const mappingsJSON = JSON.stringify(currentState.mappings);
    const gamepadJSON = JSON.stringify(s);

    if (gamepadJSON !== lastRenderedGamepadJSON || mappingsJSON !== lastRenderedMappingsJSON || window._forceRedraw) {
        drawVisualizer(s, currentState.mappings);
        lastRenderedGamepadJSON = gamepadJSON;
        lastRenderedMappingsJSON = mappingsJSON;
        window._forceRedraw = false;
    }

    if (currentState.debug_active && ui.debugPanel.style.display !== 'none') {
        // Hex Grid
        const report = currentState.raw_report; // Array
        const len = report.length;
        for (let i = 0; i < 80; i++) {
            if (i >= len) break;
            const val = report[i];
            const cell = document.getElementById('hex-' + i);
            if (cell) {
                const hex = val.toString(16).toUpperCase().padStart(2, '0');
                if (!cell.innerHTML.startsWith(hex)) {
                    cell.innerHTML = `${hex}<span>${i}</span>`;
                    cell.className = 'hex-cell ' + (val > 0 ? 'nonzero' : '');
                }
            }
        }

        const mode = report[0];
        const modeText = mode === 0x01 ? '(Simple 0x01)' : (mode === 0x31 ? '(Native 0x31)' : `(Unk 0x${mode.toString(16)})`);
        setText(ui.reportMode, modeText);

        setText(ui.btnFuzzer, currentState.fuzzer_active ? "STOP Fuzzing" : "START Auto-Discovery");
        setText(ui.fuzzerStatus, currentState.fuzzer_log);
        setText(ui.devicePath, currentState.device_path_str);
        setText(ui.lastWrite, currentState.last_write_status);
        setText(ui.packetHex, currentState.last_packet_hex);
        setText(ui.btnSweep, currentState.sweep_active ? "STOP Sweep" : "Start RGB Sweep");
        setText(ui.logDevices, currentState.detected_devices_log);
        setText(ui.logProto, currentState.protocol_log);
    }
}

function syncManualInputs(s) {
    if (!s) return;
    ui.inpRepId.value = s.manual_report_id.toString(16).toUpperCase().padStart(2, '0');
    ui.inpFlagOff.value = s.manual_flag_offset;
    ui.inpRgbOff.value = s.manual_rgb_offset;
    ui.inpPled.value = s.manual_player_led;
    ui.inpPledBright.value = s.manual_pled_bright;
    ui.inpPledBrightOff.value = s.manual_pled_bright_off;
    ui.inpBtFlags.value = s.bt_flag_val.toString(16).toUpperCase().padStart(2, '0');
    ui.inpBtFlags2.value = s.bt_flag_val2.toString(16).toUpperCase().padStart(2, '0');
    ui.inpBtLen.value = s.manual_bt_len;
    ui.chkFeature.checked = s.send_as_feature;
    ui.sldR.value = s.manual_r;
    ui.sldG.value = s.manual_g;
    ui.sldB.value = s.manual_b;
}

// --- Canvas Drawing Logic ---

function getBatteryLedMask(battery) {
    if (battery >= 90) return 0x1F;      // 5 LEDs
    if (battery >= 70) return 0x0F;      // 4 LEDs
    if (battery >= 50) return 0x07;      // 3 LEDs
    if (battery >= 30) return 0x03;      // 2 LEDs
    if (battery >= 10) return 0x01;      // 1 LED
    return 0x00;
}

function drawVisualizer(s, mappings) {
    ctx.clearRect(0, 0, canvas.width, canvas.height);

    const imgW = 1669;
    const imgH = 1046;

    const availableW = canvas.width - 80;
    let scale = availableW / imgW;

    if (scale > 0.6) scale = 0.6;

    const groupHeight = 1080 * scale + 70;
    if (groupHeight > canvas.height - 40) {
        scale = (canvas.height - 110) / 1080;
    }

    // Offset visualizer slightly down
    const offsetY = (canvas.height / 2) - (525 * scale - 35) + 7;
    const offsetX = (canvas.width - imgW * scale) / 2;

    const indY1 = offsetY - 70; // L2/R2
    const indY2 = offsetY - 35; // L1/R1

    layoutData = { offsetX, offsetY, scale };

    const p = (x, y) => ({ x: offsetX + x * scale, y: offsetY + y * scale });
    const sz = (v) => v * scale;

    // --- 1. Draw RGB Backlight (Behind Controller) ---
    if (currentState) {
        let r = parseInt(ui.mainSldR.value);
        let g = parseInt(ui.mainSldG.value);
        let b = parseInt(ui.mainSldB.value);
        
        const bright = parseInt(ui.mainSldBright.value) / 255.0; 
        
        const applyVisualScaling = (val, b) => {
            if (val === 0 || b === 0) return 0;
            let v = (val / 255.0) * b;
            v = Math.pow(v, 0.5);
            v = 0.1 + v * 0.9;
            return Math.round(v * 255);
        };

        const vr = applyVisualScaling(r, bright);
        const vg = applyVisualScaling(g, bright);
        const vb = applyVisualScaling(b, bright);

        ctx.fillStyle = `rgb(${vr}, ${vg}, ${vb})`;
        const rgbRect = { x: 835, y: 240, w: 700, h: 350 };
        const scRect = {
            x: p(rgbRect.x, rgbRect.y).x - sz(rgbRect.w)/2,
            y: p(rgbRect.x, rgbRect.y).y - sz(rgbRect.h)/2,
            w: sz(rgbRect.w),
            h: sz(rgbRect.h)
        };
        
        ctx.save();
        ctx.fillRect(scRect.x, scRect.y, scRect.w, scRect.h);
        ctx.restore();
    }

    // --- 2. Draw Controller Image ---
    if (bgImage.complete) {
        ctx.drawImage(bgImage, offsetX, offsetY, imgW * scale, imgH * scale);
    }

    // --- 3. Draw Player LEDs ---
    if (ledImage.complete && currentState) {
        let mask = 0x00;
        if (currentState.show_battery_led) {
            mask = getBatteryLedMask(s.battery);
        } else {
            mask = 0x04; 
        }

        const ledY = 378; 
        const spacing = 25; 
        
        const ledPositions = [
            { id: 0, bit: 0x01, offset: -7 }, // Left Outer
            { id: 1, bit: 0x02, offset: -2 }, // Left Inner
            { id: 4, bit: 0x10, offset:  7 }, // Right Outer
            { id: 3, bit: 0x08, offset:  2 }, // Right Inner
            { id: 2, bit: 0x04, offset:  0 }, // Center (Last = Top)
        ];

        ledPositions.forEach(led => {
            if ((mask & led.bit) !== 0) {
                const cx = 835 + (led.offset * spacing);
                const cy = ledY;
                
                const w = 84;
                const h = 12;
                
                const pos = p(cx, cy);
                const sw = sz(w);
                const sh = sz(h);
                
                ctx.drawImage(ledImage, pos.x - sw/2, pos.y - sh/2, sw, sh);
            }
        });
    }

    const cActive = 'rgba(0, 255, 128, 0.7)';
    const cModified = 'rgba(198, 120, 221, 0.6)';
    const cInactiveBg = 'rgba(40, 40, 40, 0.9)';
    const cx = canvas.width / 2;
    const colOffset = 220;

    // Helpers
    const rect = (x, y, w, h, active, key) => {
        const m = mappings && mappings.find(x => x.source === key);
        const isMod = m && isMappingModified(m);

        if (isMod) {
            ctx.strokeStyle = '#c678dd';
            ctx.lineWidth = 2;
            ctx.strokeRect(p(x, y).x - sz(w) / 2 - 2, p(x, y).y - sz(h) / 2 - 2, sz(w) + 4, sz(h) + 4);
        }

        if (active) {
            ctx.fillStyle = cActive;
            ctx.fillRect(p(x, y).x - sz(w) / 2, p(x, y).y - sz(h) / 2, sz(w), sz(h));
        }
        if (hoveredButton === key) {
            ctx.strokeStyle = '#61afef';
            ctx.lineWidth = 2;
            ctx.strokeRect(p(x, y).x - sz(w) / 2, p(x, y).y - sz(h) / 2, sz(w), sz(h));
        }
    };

    const circle = (x, y, r, active, key) => {
        const m = mappings && mappings.find(x => x.source === key);
        const isMod = m && isMappingModified(m);

        if (isMod) {
            ctx.beginPath();
            ctx.arc(p(x, y).x, p(x, y).y, sz(r) + 2, 0, Math.PI * 2);
            ctx.strokeStyle = '#c678dd';
            ctx.lineWidth = 2;
            ctx.stroke();
        }

        if (active) {
            ctx.beginPath();
            ctx.arc(p(x, y).x, p(x, y).y, sz(r), 0, Math.PI * 2);
            ctx.fillStyle = cActive;
            ctx.fill();
        }
        if (hoveredButton === key) {
            ctx.beginPath();
            ctx.arc(p(x, y).x, p(x, y).y, sz(r), 0, Math.PI * 2);
            ctx.strokeStyle = '#61afef';
            ctx.lineWidth = 2;
            ctx.stroke();
        }
    };

    // --- Floating Indicators (Classic Style) ---
    const indW = 160;
    const indH_Trig = 24;
    const indH_Btn = 20;
    const indRadius = 6;

    const drawBox = (xCenter, yTop, label, val, isAnalog, key) => {
        const h = isAnalog ? indH_Trig : indH_Btn;
        const x = xCenter - indW / 2;

        const m = mappings && mappings.find(x => x.source === key);
        if (m && isMappingModified(m)) {
            ctx.strokeStyle = '#c678dd';
            ctx.lineWidth = 2;
            ctx.beginPath();
            ctx.roundRect(x - 2, yTop - 2, indW + 4, h + 4, indRadius);
            ctx.stroke();
        }

        // Background
        ctx.fillStyle = cInactiveBg;
        ctx.beginPath();
        ctx.roundRect(x, yTop, indW, h, indRadius);
        ctx.fill();

        // Fill
        if (val > 0) {
            ctx.fillStyle = 'rgba(0, 255, 128, 0.5)';
            ctx.beginPath();
            if (isAnalog) {
                const fillW = indW * val;
                ctx.save();
                ctx.beginPath();
                ctx.roundRect(x, yTop, indW, h, indRadius);
                ctx.clip();
                ctx.fillRect(x, yTop, fillW, h);
                ctx.restore();
            } else {
                ctx.roundRect(x, yTop, indW, h, indRadius);
                ctx.fill();
            }
        }

        if (hoveredButton === key) {
            ctx.strokeStyle = '#61afef';
            ctx.lineWidth = 2;
            ctx.beginPath();
            ctx.roundRect(x, yTop, indW, h, indRadius);
            ctx.stroke();
        }

        // Text
        ctx.fillStyle = '#eee';
        ctx.font = '13px "Segoe UI", sans-serif';
        ctx.textAlign = 'center';
        ctx.textBaseline = 'middle';
        const text = isAnalog ? `${label}: ${(val * 100).toFixed(0)}%` : label;
        ctx.fillText(text, xCenter, yTop + h / 2 + 1);
    };

    // L2 / R2 (Top Row)
    drawBox(cx - colOffset, indY1, "L2", s.l2, true, 'L2');
    drawBox(cx + colOffset, indY1, "R2", s.r2, true, 'R2');

    // L1 / R1 (Bottom Row)
    drawBox(cx - colOffset, indY2, "L1", s.btn_l1 ? 1.0 : 0.0, false, 'L1');
    drawBox(cx + colOffset, indY2, "R1", s.btn_r1 ? 1.0 : 0.0, false, 'R1');


    // --- Controller Buttons ---

    // Face Buttons
    const fx = 1363, fy = 313, foff = 123, fr = 60;
    circle(fx, fy - foff, fr, s.btn_triangle, 'Triangle');
    circle(fx + foff, fy, fr, s.btn_circle, 'Circle');
    circle(fx, fy + foff, fr, s.btn_cross, 'Cross');
    circle(fx - foff, fy, fr, s.btn_square, 'Square');

    // Dpad
    const dx = 305, dy = 310, doff = 100, ds = 90;
    rect(dx, dy - doff, ds, ds, s.dpad_up, 'DpadUp');
    rect(dx, dy + doff, ds, ds, s.dpad_down, 'DpadDown');
    rect(dx - doff, dy, ds, ds, s.dpad_left, 'DpadLeft');
    rect(dx + doff, dy, ds, ds, s.dpad_right, 'DpadRight');

    // Middle Buttons
    rect(445, 130, 50, 95, s.btn_share, 'Share');
    rect(1230, 130, 50, 95, s.btn_options, 'Options');
    circle(835, 525, 50, s.btn_ps, 'PS');

    // Mute Button
    const mx = 835, my = 625, mw = 80, mh = 30;
    const mMapping = mappings && mappings.find(x => x.source === 'Mute');
    if (mMapping && isMappingModified(mMapping)) {
        ctx.strokeStyle = '#c678dd';
        ctx.lineWidth = 2;
        ctx.beginPath();
        ctx.roundRect(p(mx, my).x - sz(mw) / 2 - 2, p(mx, my).y - sz(mh) / 2 - 2, sz(mw) + 4, sz(mh) + 4, 4);
        ctx.stroke();
    }
    if (s.btn_mute) {
        ctx.fillStyle = cActive;
        ctx.beginPath();
        ctx.roundRect(p(mx, my).x - sz(mw) / 2, p(mx, my).y - sz(mh) / 2, sz(mw), sz(mh), 4);
        ctx.fill();
    }
    if (hoveredButton === 'Mute') {
        ctx.strokeStyle = '#61afef';
        ctx.lineWidth = 2;
        ctx.beginPath();
        ctx.roundRect(p(mx, my).x - sz(mw) / 2, p(mx, my).y - sz(mh) / 2, sz(mw), sz(mh), 4);
        ctx.stroke();
    }

    // --- Touchpad (Refined Shape) ---
    const drawTouchpadShape = (inset = 0, side = 'full') => {
        let c = {
            topY: 40, botY: 350,
            xTL: 510, xTR: 1160,
            xBL: 570, xBR: 1100,
            radTop: 20, sideBulge: 5, botCornerRadius: 60
        };
        
        const midX = 835;

        c.topY += inset;
        c.botY -= inset;

        if (side === 'left') {
            c.xTL += inset;
            c.xBL += inset; 
            c.xTR = midX - inset;
            c.xBR = midX - inset;
        } else if (side === 'right') {
            c.xTR -= inset;
            c.xBR -= inset;
            c.xTL = midX + inset;
            c.xBL = midX + inset;
        } else {
            c.xTL += inset;
            c.xBL += inset;
            c.xTR -= inset;
            c.xBR -= inset;
        }

        const t = 1.0 - (c.botCornerRadius / (c.botY - c.topY));
        
        let xSideEndR, xSideEndL;
        
        if (side === 'left') {
            xSideEndR = c.xTR; 
        } else {
            xSideEndR = c.xTR + (c.xBR - c.xTR) * t;
        }

        if (side === 'right') {
             xSideEndL = c.xTL;
        } else {
             xSideEndL = c.xTL + (c.xBL - c.xTL) * t;
        }

        const ySideEnd = c.botY - c.botCornerRadius;

        // Start/End points for top corners
        const tlStart = p(c.xTL + c.radTop, c.topY);
        const trStart = p(c.xTR - c.radTop, c.topY);
        const trEnd = p(c.xTR, c.topY + c.radTop);
        const tlEnd = p(c.xTL, c.topY + c.radTop);
        
        const sideEndR = p(xSideEndR, ySideEnd);
        const sideEndL = p(xSideEndL, ySideEnd);
        
        // Bottom Corner Starts
        const botRightStart = p(c.xBR - c.botCornerRadius, c.botY);
        const botLeftEnd = p(c.xBL + c.botCornerRadius, c.botY);

        ctx.beginPath();
        
        // Top Edge
        ctx.moveTo(tlStart.x, tlStart.y);
        ctx.lineTo(trStart.x, trStart.y);
        
        // Top Right Corner
        if (side === 'left') {
             // Sharp corner at split
             ctx.lineTo(p(c.xTR, c.topY).x, p(c.xTR, c.topY).y);
        } else {
             ctx.quadraticCurveTo(p(c.xTR, c.topY).x, p(c.xTR, c.topY).y, trEnd.x, trEnd.y);
        }

        // Right Side
        if (side === 'left') {
             // Straight line down
             ctx.lineTo(p(c.xBR, c.botY).x, p(c.xBR, c.botY).y);
        } else {
             // Curve
             ctx.quadraticCurveTo(p((c.xTR + xSideEndR) / 2 + c.sideBulge, (c.topY + ySideEnd) / 2).x, p((c.xTR + xSideEndR) / 2 + c.sideBulge, (c.topY + ySideEnd) / 2).y, sideEndR.x, sideEndR.y);
             // Bottom Right Corner
             ctx.quadraticCurveTo(p(c.xBR, c.botY).x, p(c.xBR, c.botY).y, botRightStart.x, botRightStart.y);
        }

        // Bottom Edge
        ctx.lineTo(botLeftEnd.x, botLeftEnd.y);
        
        // Bottom Left Corner & Left Side
        if (side === 'right') {
            // Sharp corner at split bottom-left
            ctx.lineTo(p(c.xBL, c.botY).x, p(c.xBL, c.botY).y);
            // Straight line up
            ctx.lineTo(p(c.xTL, c.topY).x, p(c.xTL, c.topY).y);
        } else {
             // Curve
             ctx.quadraticCurveTo(p(c.xBL, c.botY).x, p(c.xBL, c.botY).y, sideEndL.x, sideEndL.y);
             ctx.quadraticCurveTo(p((c.xTL + xSideEndL) / 2 - c.sideBulge, (c.topY + ySideEnd) / 2).x, p((c.xTL + xSideEndL) / 2 - c.sideBulge, (c.topY + ySideEnd) / 2).y, tlEnd.x, tlEnd.y);
             // Top Left Corner
             ctx.quadraticCurveTo(p(c.xTL, c.topY).x, p(c.xTL, c.topY).y, tlStart.x, tlStart.y);
        }
        
        ctx.closePath();
    };

    const tpMapping = mappings && mappings.find(x => x.source === 'Touchpad');
    if (tpMapping && isMappingModified(tpMapping)) {
        drawTouchpadShape(0, 'full');
        ctx.strokeStyle = '#c678dd';
        ctx.lineWidth = 2;
        ctx.stroke();
    }

    if (hoveredButton === 'Touchpad') {
        drawTouchpadShape(0, 'full');
        ctx.strokeStyle = '#61afef';
        ctx.lineWidth = 2;
        ctx.stroke();
    }

    // --- Touchpad Split Zones (Inscribed Frames) ---
    const drawSplitZone = (key, side) => {
        const m = mappings && mappings.find(x => x.source === key);
        const isMod = m && isMappingModified(m);
        const isHover = hoveredButton === key;
        
        // Determine active state
        let isActive = false;
        if (s.btn_touchpad && s.touch_active) {
            if (key === 'TouchpadLeft' && s.touch_x < 960) isActive = true;
            if (key === 'TouchpadRight' && s.touch_x >= 960) isActive = true;
        }

        // Draw only if relevant
        if (isMod || isHover || isActive) {
            drawTouchpadShape(15, side); // 15px inset
            
            if (isActive) {
                ctx.fillStyle = cActive;
                ctx.fill();
            }
            if (isMod) {
                ctx.strokeStyle = '#c678dd';
                ctx.lineWidth = 2;
                ctx.stroke();
            }
            if (isHover) {
                ctx.strokeStyle = '#61afef';
                ctx.lineWidth = 2;
                ctx.stroke();
            }
        }
    };

    drawSplitZone('TouchpadLeft', 'left');
    drawSplitZone('TouchpadRight', 'right');

    // --- Sticks ---
    const lsx = 565, rsx = 1105, sy = 535, strav = 55, srad = 110;
    const drawStick = (cx, cy, xval, yval, click, key) => {
        const center = p(cx, cy);
        const stickPos = {
            x: center.x + xval * sz(strav),
            y: center.y + yval * sz(strav)
        };
        const d = sz(srad * 2);

        if (stickImage.complete) {
            ctx.drawImage(stickImage, stickPos.x - d / 2, stickPos.y - d / 2, d, d);
            if (click) {
                // Overlay for click
                ctx.globalCompositeOperation = 'source-atop';
                ctx.fillStyle = 'rgba(0, 255, 128, 0.4)';
                ctx.beginPath();
                ctx.arc(stickPos.x, stickPos.y, d / 2, 0, Math.PI * 2);
                ctx.fill();
                ctx.globalCompositeOperation = 'source-over';
            }
        } else {
            ctx.beginPath();
            ctx.arc(stickPos.x, stickPos.y, d / 2, 0, Math.PI * 2);
            ctx.fillStyle = click ? cActive : '#444';
            ctx.fill();
        }

        const stickKey = (key === 'L3' ? 'LeftStick' : 'RightStick');
        const isStickHover = (hoveredButton === stickKey);
        const isBtnHover = (hoveredButton === key);

        const mStick = mappings && mappings.find(x => x.source === stickKey);
        const mBtn = mappings && mappings.find(x => x.source === key);

        // Draw Outer Stick Highlight (Modified)
        if (mStick && isMappingModified(mStick)) {
            ctx.beginPath();
            ctx.arc(center.x, center.y, sz(100) + 2, 0, Math.PI * 2);
            ctx.strokeStyle = '#c678dd';
            ctx.lineWidth = 2;
            ctx.stroke();
        }

        // Draw Outer Stick Highlight (Axis Hover)
        if (isStickHover) {
            ctx.beginPath();
            ctx.arc(center.x, center.y, sz(100), 0, Math.PI * 2);
            ctx.strokeStyle = '#61afef';
            ctx.lineWidth = 2;
            ctx.stroke();
        }

        // Draw Inner Button Area (L3/R3)
        if (mBtn && isMappingModified(mBtn)) {
            ctx.beginPath();
            ctx.arc(center.x, center.y, sz(50) + 2, 0, Math.PI * 2);
            ctx.strokeStyle = '#c678dd';
            ctx.lineWidth = 2;
            ctx.stroke();
        }

        if (isBtnHover) {
            ctx.beginPath();
            ctx.arc(center.x, center.y, sz(50), 0, Math.PI * 2);
            ctx.strokeStyle = '#61afef';
            ctx.lineWidth = 3;
            ctx.fillStyle = 'rgba(97, 175, 239, 0.2)';
            ctx.fill();
            ctx.stroke();
        }
    };

    drawStick(lsx, sy, s.left_x, s.left_y, s.btn_l3, 'L3');
    drawStick(rsx, sy, s.right_x, s.right_y, s.btn_r3, 'R3');

    const canvX = canvas.offsetLeft;
    const canvY = canvas.offsetTop;

    // Position Main Profile Selector (Aligned with indicators)
    ui.mainProfileCtrl.style.top = `${canvY + indY1 - 70}px`;

    // Position Deadzone Controls (Relative to wrapper, so add canvas offsets)
    // Width = 160px. Half = 80px.
    const posL = p(lsx, sy + 220);
    const posR = p(rsx, sy + 220);

    ui.dzLeft.style.left = `${canvX + posL.x + 10 - 65}px`; 
    ui.dzLeft.style.top = `${canvY + posL.y}px`;
    ui.dzRight.style.left = `${canvX + posR.x - 10 - 95}px`;
    ui.dzRight.style.top = `${canvY + posR.y}px`;

    // Position Touchpad Sens Slider
    // Just above the touchpad. Touchpad top is at: offsetY + 45 * scale
    const tpTopY = offsetY + 45 * scale;
    // Centered at cx. Width is 230px, so offset is 115px.
    ui.sensTouchCtrl.style.left = `${canvX + cx - 115}px`; 
    ui.sensTouchCtrl.style.top = `${canvY + tpTopY - 85}px`;

    // Position Trigger Controls (Above L2/R2 indicators)
    // Indicators are 160px wide, centered at cx +/- colOffset
    // We align our 160px widgets exactly at the same center
    ui.triggerL2Ctrl.style.left = `${canvX + cx - colOffset - 80}px`;
    ui.triggerL2Ctrl.style.top = `${canvY + indY1 - 50}px`;
    ui.triggerR2Ctrl.style.left = `${canvX + cx + colOffset - 80}px`;
    ui.triggerR2Ctrl.style.top = `${canvY + indY1 - 50}px`;

    // Position Vertical Trigger Sliders
    const l2StartBox = el('trigger-l2-start-box');
    const l2ForceBox = el('trigger-l2-force-box');
    const r2StartBox = el('trigger-r2-start-box');
    const r2ForceBox = el('trigger-r2-force-box');

    // Position them higher up, near the triggers
    const sliderTopY = canvY + indY1 - 50; 

    // Left Side
    const leftEdge = canvX + offsetX;
    if(l2StartBox) {
        l2StartBox.style.left = `${leftEdge - 10}px`; // Outer
        l2StartBox.style.top = `${sliderTopY}px`;
    }
    if(l2ForceBox) {
        l2ForceBox.style.left = `${leftEdge + 20}px`; // Inner
        l2ForceBox.style.top = `${sliderTopY}px`; // Top aligned with Start
    }

    // Right Side
    const rightEdge = canvX + offsetX + imgW * scale;
    if(r2StartBox) {
        r2StartBox.style.left = `${rightEdge - 10}px`; // Outer (Mirrored -10)
        r2StartBox.style.top = `${sliderTopY}px`;
    }
    if(r2ForceBox) {
        r2ForceBox.style.left = `${rightEdge - 40}px`; // Inner (Mirrored -40)
        r2ForceBox.style.top = `${sliderTopY}px`; // Top aligned with Start
    }

    // Position RGB Widget (Bottom Center of Controller)
    // Increased gap from deadzone controls
    const posRGB = p(imgW / 2, 1000);
    ui.rgbCtrl.style.left = `${canvX + posRGB.x}px`;
    ui.rgbCtrl.style.top = `${canvY + posRGB.y}px`;

    // Show/Hide Sensitivity sliders if mouse move is mapped
    const hasMouseMove = (source) => {
        const m = mappings && mappings.find(x => x.source === source);
        return m && m.targets.some(t => t.MouseMove !== undefined);
    };

    ui.boxSensLeft.style.display = hasMouseMove('LeftStick') ? 'flex' : 'none';
    ui.boxSensRight.style.display = hasMouseMove('RightStick') ? 'flex' : 'none';
    ui.sensTouchCtrl.style.display = hasMouseMove('Touchpad') ? 'flex' : 'none';
}

// Main RGB Logic
const updateMainRgb = () => {
    const r = parseInt(ui.mainSldR.value);
    const g = parseInt(ui.mainSldG.value);
    const b = parseInt(ui.mainSldB.value);
    const bright = parseInt(ui.mainSldBright.value);

    updateMainColorPreview();
    
    // Optimistic UI Update: Update local state immediately for the canvas
    if (currentState) {
        currentState.rgb_r = r;
        currentState.rgb_g = g;
        currentState.rgb_b = b;
        currentState.rgb_brightness = bright;
        window._forceRedraw = true; // Force redraw loop to render this frame
        hasNewState = true;
    }

    // Send unscaled values, backend handles scaling for the controller
    invoke('set_rgb', { r, g, b, brightness: bright });
};

function updateMainColorPreview() {
    const r = parseInt(ui.mainSldR.value);
    const g = parseInt(ui.mainSldG.value);
    const b = parseInt(ui.mainSldB.value);
    const bright = parseInt(ui.mainSldBright.value) / 255.0;
    
    const applyVisualScaling = (val, b) => {
        if (val === 0 || b === 0) return 0;
        let v = (val / 255.0) * b;
        v = Math.pow(v, 0.5);
        v = 0.1 + v * 0.9;
        return Math.round(v * 255);
    };

    const vr = applyVisualScaling(r, bright);
    const vg = applyVisualScaling(g, bright);
    const vb = applyVisualScaling(b, bright);

    // Show the visually corrected color in preview
    ui.mainColorPreview.style.background = `rgb(${vr}, ${vg}, ${vb})`;
}

[ui.mainSldR, ui.mainSldG, ui.mainSldB, ui.mainSldBright].forEach(el => {
    el.addEventListener('input', updateMainColorPreview);
    el.addEventListener('change', updateMainRgb);
});

ui.selPledBright.addEventListener('change', (e) => {
    invoke('set_player_led_brightness', { val: parseInt(e.target.value) });
});

ui.chkBatLed.addEventListener('change', (e) => {
    if (currentState) {
        currentState.show_battery_led = e.target.checked;
        window._forceRedraw = true;
        hasNewState = true;
    }
    invoke('set_show_battery_led', { val: e.target.checked });
});

// Deadzone Event Listeners
const updateDeadzones = (e) => {
    let l, r;
    if (e.target.id.includes('left')) {
        l = parseFloat(e.target.value);
        ui.sldDzLeft.value = l;
        ui.inpDzLeft.value = l;
        r = parseFloat(ui.sldDzRight.value);
    } else {
        r = parseFloat(e.target.value);
        ui.sldDzRight.value = r;
        ui.inpDzRight.value = r;
        l = parseFloat(ui.sldDzLeft.value);
    }
    
    if (currentState) {
        currentState.deadzone_left = l;
        currentState.deadzone_right = r;
        window._forceRedraw = true;
        hasNewState = true;
    }

    invoke('set_deadzones', { left: l, right: r });
};
ui.sldDzLeft.addEventListener('input', updateDeadzones);
ui.inpDzLeft.addEventListener('change', updateDeadzones);
ui.sldDzRight.addEventListener('input', updateDeadzones);
ui.inpDzRight.addEventListener('change', updateDeadzones);

const updateSens = (e) => {
    let l, r;
    if (e.target.id.includes('left')) {
        l = parseFloat(e.target.value);
        ui.sldSensLeft.value = l;
        ui.inpSensLeft.value = l;
        r = parseFloat(ui.sldSensRight.value);
    } else {
        r = parseFloat(e.target.value);
        ui.sldSensRight.value = r;
        ui.inpSensRight.value = r;
        l = parseFloat(ui.sldSensLeft.value);
    }
    invoke('set_mouse_sens', { left: l, right: r });
};
ui.sldSensLeft.addEventListener('input', updateSens);
ui.inpSensLeft.addEventListener('change', updateSens);
ui.sldSensRight.addEventListener('input', updateSens);
ui.inpSensRight.addEventListener('change', updateSens);

const updateSensTouch = (e) => {
    const val = parseFloat(e.target.value);
    ui.sldSensTouch.value = val;
    ui.inpSensTouch.value = val;
    invoke('set_touchpad_sens', { sens: val });
};
ui.sldSensTouch.addEventListener('input', updateSensTouch);
ui.inpSensTouch.addEventListener('change', updateSensTouch);

// Adaptive Triggers
const updateTriggerL2 = () => {
    const mode = parseInt(ui.selTriggerL2Mode.value);
    const start = parseInt(ui.sldTriggerL2Start.value);
    const force = parseInt(ui.sldTriggerL2Force.value);
    const display = (mode === 0) ? 'none' : 'flex';
    el('trigger-l2-start-box').style.display = display;
    el('trigger-l2-force-box').style.display = display;
    invoke('set_trigger_l2', { mode, start, force });
};

const updateTriggerR2 = () => {
    const mode = parseInt(ui.selTriggerR2Mode.value);
    const start = parseInt(ui.sldTriggerR2Start.value);
    const force = parseInt(ui.sldTriggerR2Force.value);
    const display = (mode === 0) ? 'none' : 'flex';
    el('trigger-r2-start-box').style.display = display;
    el('trigger-r2-force-box').style.display = display;
    invoke('set_trigger_r2', { mode, start, force });
};

[ui.selTriggerL2Mode, ui.sldTriggerL2Start, ui.sldTriggerL2Force].forEach(el => {
    el.addEventListener('change', updateTriggerL2);
    if (el.type === 'range') el.addEventListener('input', updateTriggerL2);
});
[ui.selTriggerR2Mode, ui.sldTriggerR2Start, ui.sldTriggerR2Force].forEach(el => {
    el.addEventListener('change', updateTriggerR2);
    if (el.type === 'range') el.addEventListener('input', updateTriggerR2);
});

window.addEventListener('resize', () => {
    if (currentState) {
        window._forceRedraw = true;
        hasNewState = true;
    }
});

// Disable Right-Click Context Menu (Production polish)
window.addEventListener('contextmenu', (e) => {
    // Optional: Allow context menu only on specific inputs if needed (e.g., for copy/paste)
    // But for this controller app, we disable it globally.
    if (!e.target.matches('input[type="text"]')) {
        e.preventDefault();
    }
});
