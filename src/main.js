const { invoke } = window.__TAURI__.core;

let currentNote = null;
let timerInterval = null;
let currentState = null;

// Available note card colors
const noteColors = ['blue', 'lilac', 'mint', 'cream', 'pink', 'sand'];

function getRandomNoteColor() {
    return noteColors[Math.floor(Math.random() * noteColors.length)];
}



async function initApp() {
    try {
        await loadAllNotes();
        await initTimer();
        setupEventListeners();
    } catch (error) {
        console.error('Failed to initialize app:', error);
        showNotification('Failed to initialize app', 'error');
    }
}

document.addEventListener('DOMContentLoaded', () => {
    setTimeout(initApp, 100);
});


function setupEventListeners() {
    // Add new note button
    const addBtn = document.querySelector('button.btn-primary');
    if (addBtn) {
        addBtn.addEventListener('click', createNote);
    }

    // Close note detail
    document.querySelectorAll('[data-note-close]').forEach(btn => {
        btn.addEventListener('click', closeNoteDetail);
    });

    // Edit note button
    const editBtn = document.querySelector('[data-note-edit]');
    if (editBtn) {
        editBtn.addEventListener('click', toggleNoteEdit);
    }

    // Subnote form submission
    document.addEventListener('submit', function (event) {
        if (event.target.id === 'subnote-form') {
            addSubnote(event);
        }
    });

    // Note card clicks
    document.addEventListener('click', function (event) {
        const noteCard = event.target.closest('.note-card');
        if (noteCard && !event.target.closest('.note-menu-btn') && !event.target.closest('.note-menu')) {
            const noteId = noteCard.getAttribute('data-note-id');
            if (noteId) {
                openNoteDetail(noteId).catch(error => {
                    console.error('Error opening note detail:', error);
                });
            }
        }
    });

    // Close menus when clicking elsewhere
    document.addEventListener('click', (event) => {
        if (!event.target.closest('.note-menu') && !event.target.closest('.note-menu-btn')) {
            document.querySelectorAll('.note-card.has-open-menu').forEach(card => {
                card.classList.remove('has-open-menu');
            });
        }
    });

    // Close note detail with Escape
    document.addEventListener('keydown', (event) => {
        if (event.key === 'Escape') {
            const noteDetail = document.querySelector('.note-detail');
            if (noteDetail && noteDetail.getAttribute('aria-hidden') === 'false') {
                closeNoteDetail();
            }
        }
    });

    // === POMODORO EVENT LISTENERS ===
    const playBtn = document.querySelector('.btn-circle--primary');
    const resetBtn = document.querySelector('.btn-circle--ghost');
    const settingsBtn = document.querySelector('.pomodoro__settings-btn');
    const settingsForm = document.getElementById('pomodoro-settings');
    const workTab = document.querySelector('[data-timer-tab="work"]');
    const breakTab = document.querySelector('[data-timer-tab="break"]');

    if (playBtn) playBtn.addEventListener('click', toggleTimer);
    if (resetBtn) resetBtn.addEventListener('click', handleReset);
    if (settingsBtn) settingsBtn.addEventListener('click', toggleSettings);
    if (settingsForm) settingsForm.addEventListener('submit', handleSettingsSubmit);
    if (workTab) workTab.addEventListener('click', () => switchTab('work'));
    if (breakTab) breakTab.addEventListener('click', () => switchTab('break'));

    // Volume slider
    const volumeSlider = document.getElementById('alarm-volume');
    const volumeValue = document.getElementById('alarm-volume-value');
    if (volumeSlider && volumeValue) {
        volumeSlider.addEventListener('input', (e) => {
            volumeValue.textContent = e.target.value;
        });
    }
}


async function loadAllNotes() {
    try {
        const notes = await invoke('get_notes');
        renderNotes(notes);
    } catch (error) {
        console.error('Failed to load notes:', error);
        showNotification('Failed to load notes', 'error');
    }
}

async function renderNotes(notes) {
    const notesGrid = document.querySelector('.notes-grid');
    if (!notesGrid) return;

    notesGrid.innerHTML = '';

    for (const note of notes) {
        const noteElement = await createNoteElement(note);
        notesGrid.appendChild(noteElement);
    }
}

async function createNoteElement(note) {
    const article = document.createElement('article');
    article.className = `note-card note-card--${getRandomNoteColor()}`;
    article.setAttribute('data-note-id', note.id);

    let childCountHtml = '';
    if (note.child_count > 0) {
        childCountHtml = `<div class="note-card__sub-notes">📌 ${note.child_count} sub-note${note.child_count > 1 ? 's' : ''}</div>`;
    }

    article.innerHTML = `
        <button type="button" class="note-menu-btn" aria-label="Opsi catatan">⋮</button>
        <div class="note-menu">
            <button type="button" class="note-menu__item" data-action="download">
                <span class="note-menu__icon">
                    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="18" height="18" fill="currentColor">
                        <path d="M3 19H21V21H3V19ZM13 9H20L12 17L4 9H11V1H13V9Z"></path>
                    </svg>
                </span>
                <span>Unduh</span>
            </button>
            <button type="button" class="note-menu__item note-menu__item--danger" data-action="delete">
                <span class="note-menu__icon">
                    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="18" height="18" fill="currentColor">
                        <path d="M17 6H22V8H20V21C20 21.5523 19.5523 22 19 22H5C4.44772 22 4 21.5523 4 21V8H2V6H7V3C7 2.44772 7.44772 2 8 2H16C16.5523 2 17 2.44772 17 3V6ZM9 11V17H11V11H9ZM13 11V17H15V11H13ZM9 4V6H15V4H9Z"></path>
                    </svg>
                </span>
                <span>Hapus</span>
            </button>
        </div>
        <h2>${escapeHtml(note.title)}</h2>
        <p>${escapeHtml(note.content_preview)}</p>
        ${childCountHtml}
    `;

    const menuBtn = article.querySelector('.note-menu-btn');
    if (menuBtn) {
        menuBtn.addEventListener('click', function (e) {
            e.stopPropagation();
            toggleNoteMenu(e);
        });
    }

    const deleteBtn = article.querySelector('[data-action="delete"]');
    if (deleteBtn) {
        deleteBtn.addEventListener('click', async (e) => {
            e.stopPropagation();
            await deleteNote(note.id);
        });
    }

    return article;
}

async function createNote() {
    try {
        const newNote = await invoke('create_note', {
            title: 'New Note',
            content: 'Start writing your note here...'
        });

        await loadAllNotes();
        openNoteDetail(newNote.id).catch(error => {
            console.error('Error opening new note detail:', error);
        });
    } catch (error) {
        console.error('Failed to create note:', error);
        showNotification('Failed to create note', 'error');
    }
}

async function openNoteDetail(noteId) {
    try {
        const note = await invoke('get_note', { id: noteId });
        currentNote = note;

        document.getElementById('detail-title').textContent = note.title;
        document.getElementById('detail-description').textContent = note.content;
        document.getElementById('detail-title-input').value = note.title;
        document.getElementById('detail-description-input').value = note.content;

        const subnoteList = document.getElementById('subnote-list');
        if (subnoteList) {
            subnoteList.innerHTML = '';
        }

        await createSubnotesFromCheckboxes(noteId, note.content);

        const noteDetail = document.querySelector('.note-detail');
        noteDetail.classList.add('is-active');
        noteDetail.setAttribute('aria-hidden', 'false');
        noteDetail.style.display = 'block';

        toggleEditState(false);
    } catch (error) {
        console.error('Failed to open note detail:', error);
        showNotification('Failed to open note', 'error');
    }
}

function closeNoteDetail() {
    const noteDetail = document.querySelector('.note-detail');
    noteDetail.classList.remove('is-active');
    noteDetail.setAttribute('aria-hidden', 'true');
    noteDetail.style.display = 'none';

    const subnoteList = document.getElementById('subnote-list');
    if (subnoteList) {
        subnoteList.innerHTML = '';
    }

    currentNote = null;
}

function toggleNoteEdit() {
    const isEditing = document.querySelector('[data-note-edit]').classList.contains('is-editing');

    if (isEditing) {
        saveCurrentNote();
    } else {
        toggleEditState(true);
    }
}

function toggleEditState(isEditing) {
    const editBtn = document.querySelector('[data-note-edit]');
    const titleView = document.getElementById('detail-title');
    const titleInput = document.getElementById('detail-title-input');
    const descView = document.getElementById('detail-description');
    const descInput = document.getElementById('detail-description-input');

    if (isEditing) {
        editBtn.classList.add('is-editing');
        titleView.style.display = 'none';
        titleInput.style.display = 'block';
        descView.style.display = 'none';
        descInput.style.display = 'block';
        editBtn.querySelector('.icon--edit').style.display = 'none';
        editBtn.querySelector('.icon--save').style.display = 'inline';
    } else {
        editBtn.classList.remove('is-editing');
        titleView.style.display = 'block';
        titleInput.style.display = 'none';
        descView.style.display = 'block';
        descInput.style.display = 'none';
        editBtn.querySelector('.icon--edit').style.display = 'inline';
        editBtn.querySelector('.icon--save').style.display = 'none';
    }
}

async function saveCurrentNote() {
    if (!currentNote) return;

    try {
        const updatedNote = await invoke('update_note', {
            id: currentNote.id,
            title: document.getElementById('detail-title-input').value,
            content: document.getElementById('detail-description-input').value
        });

        currentNote = updatedNote;

        const noteElement = document.querySelector(`[data-note-id="${currentNote.id}"]`);
        if (noteElement) {
            noteElement.querySelector('h2').textContent = updatedNote.title;
            noteElement.querySelector('p').textContent = updatedNote.content;
        }

        document.getElementById('detail-title').textContent = updatedNote.title;
        document.getElementById('detail-description').textContent = updatedNote.content;

        showNotification('Note saved successfully', 'success');
        toggleEditState(false);
    } catch (error) {
        console.error('Failed to save note:', error);
        showNotification('Failed to save note', 'error');
    }
}

async function deleteNote(noteId) {
    if (!confirm('Are you sure you want to delete this note?')) {
        return;
    }

    try {
        await invoke('delete_note', { id: noteId });
        await loadAllNotes();

        if (currentNote && currentNote.id === noteId) {
            closeNoteDetail();
        }

        showNotification('Note deleted successfully', 'success');
    } catch (error) {
        console.error('Failed to delete note:', error);
        showNotification('Failed to delete note', 'error');
    }
}

async function addSubnote(event) {
    event.preventDefault();

    if (!currentNote) return;

    const input = event.target.querySelector('input');
    const subnoteText = input.value.trim();

    if (!subnoteText) return;

    input.value = '';

    try {
        const newCheckbox = `- [ ] ${subnoteText}`;
        const updatedContent = currentNote.content
            ? currentNote.content + '\n' + newCheckbox
            : newCheckbox;

        const updatedNote = await invoke('update_note', {
            id: currentNote.id,
            title: currentNote.title,
            content: updatedContent
        });

        currentNote = updatedNote;

        document.getElementById('detail-description').textContent = updatedNote.content;
        document.getElementById('detail-description-input').value = updatedNote.content;

        await createSubnotesFromCheckboxes(currentNote.id, updatedNote.content);

        showNotification('Subnote added successfully', 'success');
    } catch (error) {
        console.error('Failed to add subnote:', error);
        showNotification('Failed to add subnote', 'error');
    }
}

async function createSubnotesFromCheckboxes(parentId, content) {
    try {
        const checkboxes = await invoke('parse_checkboxes', { content });

        if (checkboxes.length === 0) {
            const subnoteList = document.getElementById('subnote-list');
            if (subnoteList) {
                subnoteList.innerHTML = '<li class="no-subnotes">No subnotes yet</li>';
            }
            return;
        }

        renderCheckboxes(parentId, checkboxes);
    } catch (error) {
        console.error('Failed to create subnotes from checkboxes:', error);
    }
}

function renderCheckboxes(parentId, checkboxes) {
    const subnoteList = document.getElementById('subnote-list');
    if (!subnoteList) return;

    subnoteList.innerHTML = '';

    checkboxes.forEach((checkbox, index) => {
        const li = document.createElement('li');
        li.className = `subnote-item${checkbox.completed ? " subnote-item--done" : ""}`;
        li.innerHTML = `
            <input type="checkbox" id="subnote-${index}" ${checkbox.completed ? "checked" : ""} />
            <label for="subnote-${index}">${escapeHtml(checkbox.text)}</label>
        `;

        const checkboxElement = li.querySelector('input[type="checkbox"]');
        if (checkboxElement) {
            checkboxElement.addEventListener('change', async (event) => {
                try {
                    const updatedNote = await invoke('update_note_checkbox_status', {
                        noteId: parentId,
                        checkboxText: checkbox.text,
                        newStatus: event.target.checked
                    });

                    currentNote = updatedNote;
                    checkbox.completed = event.target.checked;
                    li.classList.toggle("subnote-item--done", event.target.checked);
                } catch (error) {
                    console.error('Error updating subnote status:', error);
                    showNotification('Failed to update subnote status', 'error');
                }
            });
        }

        subnoteList.appendChild(li);
    });
}

function toggleNoteMenu(event) {
    event.stopPropagation();

    document.querySelectorAll('.note-card').forEach(card => {
        if (card !== event.target.closest('.note-card')) {
            card.classList.remove('has-open-menu');
        }
    });

    const noteCard = event.target.closest('.note-card');
    if (noteCard) {
        noteCard.classList.toggle('has-open-menu');
    }
}


async function initTimer() {
    currentState = await invoke('get_timer_state');
    updateDisplay();
}

function updateDisplay() {
    if (!currentState) return;

    const pomodoroTime = document.getElementById('pomodoro-time');
    const pomodoroContainer = document.querySelector('.pomodoro');
    const workTab = document.querySelector('[data-timer-tab="work"]');
    const breakTab = document.querySelector('[data-timer-tab="break"]');

    if (!pomodoroTime) return;

    const time = formatTime(currentState.remaining);
    pomodoroTime.textContent = time;

    if (currentState.is_break) {
        pomodoroContainer.classList.remove('pomodoro--work');
        pomodoroContainer.classList.add('pomodoro--break');
        breakTab.classList.add('is-active');
        workTab.classList.remove('is-active');
    } else {
        pomodoroContainer.classList.remove('pomodoro--break');
        pomodoroContainer.classList.add('pomodoro--work');
        workTab.classList.add('is-active');
        breakTab.classList.remove('is-active');
    }

    updatePlayButton();
}

function formatTime(seconds) {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${String(mins).padStart(2, '0')}:${String(secs).padStart(2, '0')}`;
}

function updatePlayButton() {
    if (!currentState) return;

    const playBtn = document.querySelector('.btn-circle--primary');
    if (!playBtn) return;

    const isRunning = !currentState.is_paused && currentState.remaining > 0;

    if (isRunning) {
        playBtn.innerHTML = `
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="18" height="18" fill="currentColor">
                <path d="M6 5H8V19H6V5ZM16 5H18V19H16V5Z"></path>
            </svg>
        `;
    } else {
        playBtn.innerHTML = `
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="18" height="18" fill="currentColor">
                <path d="M19.376 12.4161L8.77735 19.4818C8.54759 19.635 8.23715 19.5729 8.08397 19.3432C8.02922 19.261 8 19.1645 8 19.0658V4.93433C8 4.65818 8.22386 4.43433 8.5 4.43433C8.59871 4.43433 8.69522 4.46355 8.77735 4.5183L19.376 11.584C19.6057 11.7372 19.6678 12.0477 19.5146 12.2774C19.478 12.3323 19.4309 12.3795 19.376 12.4161Z"></path>
            </svg>
        `;
    }
}

async function toggleTimer() {
    if (!currentState) return;

    if (currentState.is_paused) {
        await invoke('resume_timer');
        currentState.is_paused = false;
        startTicking();
    } else {
        await invoke('pause_timer');
        currentState.is_paused = true;
        stopTicking();
    }

    updatePlayButton();
}

function startTicking() {
    if (timerInterval) return;

    timerInterval = setInterval(async () => {
        const timeStr = await invoke('tick_timer');
        currentState = await invoke('get_timer_state');
        const pomodoroTime = document.getElementById('pomodoro-time');
        if (pomodoroTime) {
            pomodoroTime.textContent = timeStr;
        }

        if (currentState.remaining === 0) {
            stopTicking();

            if (!currentState.is_break) {
                if (confirm('Work time selesai! Mulai break?')) {
                    await invoke('start_break');
                    currentState = await invoke('get_timer_state');
                    updateDisplay();
                    switchTab("work"); 
                }
            } else {
                alert('Break time selesai!')
                await invoke('reset_timer'); 
                currentState = await invoke('get_timer_state');
                updateDisplay();
            }
        }
    }, 1000);
}

function stopTicking() {
    if (timerInterval) {
        clearInterval(timerInterval);
        timerInterval = null;
    }
}

async function handleReset() {
    stopTicking();
    const timeStr = await invoke('reset_timer');
    currentState = await invoke('get_timer_state');
    const pomodoroTime = document.getElementById('pomodoro-time');
    if (pomodoroTime) {
        pomodoroTime.textContent = timeStr;
    }
    updatePlayButton();
}

function toggleSettings() {
    const settingsForm = document.getElementById('pomodoro-settings');
    if (!settingsForm) return;

    settingsForm.classList.toggle('is-open');
    settingsForm.setAttribute(
        'aria-hidden',
        !settingsForm.classList.contains('is-open')
    );
}

async function handleSettingsSubmit(e) {
    e.preventDefault();

    const workMin = parseInt(document.getElementById('work-duration').value);
    const breakMin = parseInt(document.getElementById('break-duration').value);

    stopTicking();

    currentState = await invoke('init_timer', {
        workMin,
        breakMin
    });

    updateDisplay();
    toggleSettings();
}

async function switchTab(mode) {
    stopTicking();

    if (mode === 'break' && !currentState.is_break) {
        await invoke('start_break');
    } else if (mode === 'work' && currentState.is_break) {
        await invoke('reset_timer');
    }

    currentState = await invoke('get_timer_state');
    updateDisplay();
}


function showNotification(message, type) {
    console.log(`${type}: ${message}`);
}

function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}