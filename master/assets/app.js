// === 1. MODAL LOGIC (For Log Inspector) ===

window.closeModal = function() {
    const container = document.getElementById('modal-container');
    if (container) {
        container.innerHTML = ''; 
    }
}

document.addEventListener('keydown', function(event) {
    if (event.key === "Escape") {
        window.closeModal();
        window.closeNodeModal(); // Also close node modal on Escape
    }
});


// === 2. LOG STREAM LOGIC (PAUSE/RESUME) ===

document.addEventListener('click', function(e) {
    if (e.target && e.target.id === 'pause-btn') {
        const btn = e.target;
        const container = btn.closest('[hx-trigger]');
        if (!container) return;

        if (btn.innerText === 'PAUSE') {
            container.setAttribute('hx-trigger', 'off');
            btn.innerText = 'RESUME';
            btn.classList.add('bg-green-600', 'text-white', 'border-transparent');
            btn.classList.remove('border-gray-600');
            
            const liveBadge = container.querySelector('.animate-pulse');
            if(liveBadge) {
                liveBadge.classList.remove('animate-pulse', 'text-green-500');
                liveBadge.classList.add('text-red-500');
                liveBadge.innerText = "PAUSED";
            }
        } else {
            btn.innerText = 'LOADING...';
            const url = container.getAttribute('hx-get');
            htmx.ajax('GET', url, { target: container, swap: 'outerHTML' });
        }
    }
});


// === 3. AUTO-SCROLL LOGIC ===

htmx.on('htmx:afterSwap', function(evt) {
    const target = evt.detail.target;
    const pre = target.querySelector('pre');
    if (pre) {
        pre.scrollTop = pre.scrollHeight;
    }
});


// === 4. NODE SWITCHING LOGIC ===

window.switchNode = function(url) {
    const input = document.getElementById('current-node-input');
    if (input) {
        input.value = url;
    }

    // Refresh System Stats immediately
    htmx.ajax('GET', '/view/stats', { 
        target: '#system-stats', 
        swap: 'innerHTML',
        values: { node: url } 
    });

    // Refresh Process Table immediately
    const table = document.getElementById('process-table');
    if (table) {
        const rowsUrl = `/view/rows?node=${encodeURIComponent(url)}`;
        htmx.ajax('GET', rowsUrl, { target: '#process-table', swap: 'outerHTML' });
    }
}


// === 5. NODE MANAGEMENT (MODAL) ===

// Helper to generate UUIDs (fallback for non-secure contexts)
function generateUUID() {
    if (typeof crypto !== 'undefined' && crypto.randomUUID) {
        return crypto.randomUUID();
    }
    return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, function(c) {
        var r = Math.random() * 16 | 0, v = c == 'x' ? r : (r & 0x3 | 0x8);
        return v.toString(16);
    });
}

// Expose these to window so HTML onclick="..." can find them
window.openNodeModal = function(id = null, name = '', url = '', token = '') {
    const modal = document.getElementById('node-modal');
    const title = document.getElementById('node-modal-title');
    
    document.getElementById('node-id').value = id || generateUUID();
    document.getElementById('node-name').value = name;
    document.getElementById('node-url').value = url;
    document.getElementById('node-token').value = token;
    
    title.innerText = id ? 'Edit Node' : 'Add New Node';
    modal.classList.remove('hidden');
}

window.closeNodeModal = function() {
    const modal = document.getElementById('node-modal');
    if(modal) modal.classList.add('hidden');
}

window.saveNode = function(event) {
    event.preventDefault();
    
    const id = document.getElementById('node-id').value;
    const name = document.getElementById('node-name').value;
    const url = document.getElementById('node-url').value;
    const token = document.getElementById('node-token').value;

    fetch('/api/nodes/save', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ 
            id, name, url, 
            token: token.trim() === "" ? null : token 
        })
    }).then(() => {
        window.closeNodeModal();
        // Reload to update the sidebar list
        window.location.reload();
    });
}

window.deleteNode = function(id) {
    if(!confirm("Are you sure you want to delete this node?")) return;
    
    fetch(`/api/nodes/delete/${id}`, { method: 'POST' })
    .then(() => window.location.reload());
}