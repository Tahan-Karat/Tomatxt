const { invoke } = window.__TAURI__.core;

// Application state
let currentNote = null;

// Available note card colors
const noteColors = ['blue', 'lilac', 'mint', 'cream', 'pink', 'sand'];

// Get a random note color
function getRandomNoteColor() {
        return noteColors[Math.floor(Math.random() * noteColors.length)];
}

// Wait for Tauri to be ready
async function initApp() {
        try {
                // Load all notes from the backend
                await loadAllNotes();

                // Setup event listeners
                setupEventListeners();

                // Initialize Pomodoro when the DOM is loaded
                initializePomodoro();
        } catch (error) {
                console.error('Failed to initialize app:', error);
                showNotification('Failed to initialize app', 'error');
        }
}

document.addEventListener('DOMContentLoaded', () => {
        // Small delay to ensure Tauri is ready
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

        // Convert checkboxes to subnotes button
        const convertBtn = document.getElementById('convert-checkboxes-btn');
        if (convertBtn) {
                convertBtn.addEventListener('click', async () => {
                        if (currentNote) {
                                await createSubnotesFromCheckboxes(currentNote.id, currentNote.content);
                        }
                });
        }

        // Subnote form submission - use event delegation
        document.addEventListener('submit', function (event) {
                if (event.target.id === 'subnote-form') {
                        addSubnote(event);
                }
        });

        // Note card clicks - use event delegation
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
}

async function loadAllNotes() {
        try {
                // Load all notes from the backend
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

        // Clear existing notes
        notesGrid.innerHTML = '';

        // Add each note to the grid
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
    <button type="button" class="note-menu-btn" aria-label="Opsi catatan">
        ⋮
    </button>

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

        // Add menu button functionality
        const menuBtn = article.querySelector('.note-menu-btn');
        if (menuBtn) {
                menuBtn.addEventListener('click', function (e) {
                        e.stopPropagation();
                        toggleNoteMenu(e);
                });
        }

        // Add delete functionality
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
                // Create a new note with default content
                const newNote = await invoke('create_note', {
                        title: 'New Note',
                        content: 'Start writing your note here...'
                });

                // Reload all notes to include the new one
                await loadAllNotes();

                // Open the new note in detail view
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

                // Populate the detail modal
                document.getElementById('detail-title').textContent = note.title;
                document.getElementById('detail-description').textContent = note.content;
                document.getElementById('detail-title-input').value = note.title;
                document.getElementById('detail-description-input').value = note.content;

                // Clear subnotes and recreate from note content
                const subnoteList = document.getElementById('subnote-list');
                if (subnoteList) {
                        subnoteList.innerHTML = '';
                }

                // Parse and display checkboxes from the note content
                await createSubnotesFromCheckboxes(noteId, note.content);

                // Show the detail modal
                const noteDetail = document.querySelector('.note-detail');
                noteDetail.classList.add('is-active');
                noteDetail.setAttribute('aria-hidden', 'false');
                noteDetail.style.display = 'block';

                // Reset to view mode
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

        // Clear subnotes
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

                // Update the note in the grid view
                const noteElement = document.querySelector(`[data-note-id="${currentNote.id}"]`);
                if (noteElement) {
                        noteElement.querySelector('h2').textContent = updatedNote.title;
                        noteElement.querySelector('p').textContent = updatedNote.content;
                }

                // Update detail view
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

                // Reload notes to reflect the deletion
                await loadAllNotes();

                // Close detail view if the deleted note was open
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

        // Clear input immediately for user feedback
        input.value = '';

        try {
                // Add checkbox to current note content
                const newCheckbox = `- [ ] ${subnoteText}`;
                const updatedContent = currentNote.content
                        ? currentNote.content + '\n' + newCheckbox
                        : newCheckbox;

                // Update the note with new checkbox
                const updatedNote = await invoke('update_note', {
                        id: currentNote.id,
                        title: currentNote.title,
                        content: updatedContent
                });

                // Update current note reference
                currentNote = updatedNote;

                // Update the detail view
                document.getElementById('detail-description').textContent = updatedNote.content;
                document.getElementById('detail-description-input').value = updatedNote.content;

                // Parse checkboxes and update display
                await createSubnotesFromCheckboxes(currentNote.id, updatedNote.content);

                showNotification('Subnote added successfully', 'success');
        } catch (error) {
                console.error('Failed to add subnote:', error);
                showNotification('Failed to add subnote', 'error');
        }
}

async function updateSubnotesList(parentId) {
        try {
                // Get child notes for this parent
                const childNotes = await invoke('get_child_notes', { parentId: parentId });

                // Render the child notes in the subnote list
                const subnoteList = document.getElementById('subnote-list');
                if (!subnoteList) return;

                // Clear existing subnotes
                subnoteList.innerHTML = '';

                // Add each child note to the list
                childNotes.forEach((childNote, index) => {
                        const li = document.createElement('li');
                        li.className = `subnote-item${childNote.is_done ? " subnote-item--done" : ""}`;
                        li.dataset.index = index;
                        li.innerHTML = `
        <input
            type="checkbox"
            id="subnote-${index}"
            ${childNote.is_done ? "checked" : ""}
        />
        <label for="subnote-${index}">${escapeHtml(childNote.title)}</label>

    `;

                        // Add event listener for checkbox change
                        const checkbox = li.querySelector('input[type="checkbox"]');
                        if (checkbox) {
                                checkbox.addEventListener('change', async (event) => {
                                        try {
                                                await invoke('update_note_status', {
                                                        id: childNote.id,
                                                        isDone: event.target.checked
                                                });

                                                li.classList.toggle("subnote-item--done", event.target.checked);
                                        } catch (error) {
                                                console.error('Error updating subnote status:', error);
                                        }
                                });
                        }

                        // Add event listener to edit button
                        const deleteBtn = li.querySelector('.subnote-delete-btn');
                        if (deleteBtn) {
                                deleteBtn.addEventListener('click', async (e) => {
                                        e.stopPropagation();
                                        await deleteNote(childNote.id);
                                        await updateSubnotesList(parentId);
                                });
                        }

                        subnoteList.appendChild(li);
                });

                if (childNotes.length === 0) {
                        subnoteList.innerHTML = '<li class="no-subnotes">No subnotes yet</li>';
                }
        } catch (error) {
                console.error('Failed to update subnotes list:', error);
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
                showNotification('Subnotes created from checkboxes', 'success');
        } catch (error) {
                console.error('Failed to create subnotes from checkboxes:', error);
                showNotification('Failed to create subnotes from checkboxes', 'error');
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
        <input
            type="checkbox"
            id="subnote-${index}"
            ${checkbox.completed ? "checked" : ""}
        />
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

        // Close any open menus first
        document.querySelectorAll('.note-card').forEach(card => {
                if (card !== event.target.closest('.note-card')) {
                        card.classList.remove('has-open-menu');
                }
        });

        // Toggle the current menu
        const noteCard = event.target.closest('.note-card');
        if (noteCard) {
                noteCard.classList.toggle('has-open-menu');
        }
}

// Close menus when clicking elsewhere
document.addEventListener('click', (event) => {
        if (!event.target.closest('.note-menu') && !event.target.closest('.note-menu-btn')) {
                document.querySelectorAll('.note-card.has-open-menu').forEach(card => {
                        card.classList.remove('has-open-menu');
                });
        }
});

// Close note detail with Escape key
document.addEventListener('keydown', (event) => {
        if (event.key === 'Escape') {
                const noteDetail = document.querySelector('.note-detail');
                if (noteDetail && noteDetail.getAttribute('aria-hidden') === 'false') {
                        closeNoteDetail();
                }
        }
});


function savePomodoroSettings(e) {
        if (e) {
                e.preventDefault();
        }

        const workDurationInput = document.getElementById('work-duration');
        const breakDurationInput = document.getElementById('break-duration');


}

function updatePomodoroForNote(note) {
        if (note && note.pomodoro_count !== undefined) {
                console.log(`Note has ${note.pomodoro_count} pomodoro sessions`);
        }
        initializePomodoro();
}

function showNotification(message, type) {
        console.log(`${type}: ${message}`);
}

function escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
}
