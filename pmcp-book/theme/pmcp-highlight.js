// PMCP-specific syntax highlighting and enhancements

(function() {
    'use strict';

    // Initialize PMCP highlighting when DOM is ready
    document.addEventListener('DOMContentLoaded', function() {
        initializePMCPHighlighting();
        addCodeBlockEnhancements();
        setupPlaygroundIntegration();
    });

    function initializePMCPHighlighting() {
        // Add PMCP-specific token recognition to existing highlighter
        if (typeof hljs !== 'undefined') {
            // Register PMCP keywords and patterns
            hljs.registerLanguage('pmcp-rust', function() {
                return {
                    name: 'PMCP Rust',
                    aliases: ['pmcp'],
                    keywords: {
                        keyword: 'async await fn let mut const struct impl enum trait use mod pub crate super self match if else while for loop break continue return',
                        built_in: 'Option Result Vec String HashMap Box Arc Mutex tokio serde_json',
                        literal: 'true false Some None Ok Err',
                        pmcp_types: 'Server Client ToolHandler RequestHandlerExtra Error ToolResult'
                    },
                    contains: [
                        hljs.C_LINE_COMMENT_MODE,
                        hljs.C_BLOCK_COMMENT_MODE,
                        {
                            className: 'pmcp-server',
                            begin: /\b(Server|ServerBuilder|ServerCapabilities)\b/
                        },
                        {
                            className: 'pmcp-client',
                            begin: /\b(Client|ClientBuilder|ClientCapabilities)\b/
                        },
                        {
                            className: 'pmcp-tool',
                            begin: /\b(ToolHandler|ToolInfo|CallToolResult|ToolResult)\b/
                        },
                        {
                            className: 'pmcp-transport',
                            begin: /\b(WebSocketTransport|HttpTransport|StreamableHttp)\b/
                        },
                        {
                            className: 'pmcp-error',
                            begin: /\b(Error|ErrorCode|ValidationError)\b/
                        }
                    ]
                };
            });
        }

        // Highlight PMCP-specific elements in existing code blocks
        document.querySelectorAll('code.language-rust').forEach(function(block) {
            highlightPMCPTokens(block);
        });
    }

    function highlightPMCPTokens(block) {
        let html = block.innerHTML;
        
        // Highlight PMCP-specific patterns
        const patterns = [
            {
                pattern: /\b(Server|Client)::builder\(\)/g,
                replacement: '<span class="pmcp-builder">$1::builder()</span>'
            },
            {
                pattern: /\.tool\(/g,
                replacement: '<span class="pmcp-method">.tool(</span>'
            },
            {
                pattern: /\.resource\(/g,
                replacement: '<span class="pmcp-method">.resource(</span>'
            },
            {
                pattern: /\.run_stdio\(\)/g,
                replacement: '<span class="pmcp-transport">.run_stdio()</span>'
            },
            {
                pattern: /pmcp::Error::(validation|internal|protocol)/g,
                replacement: '<span class="pmcp-error">pmcp::Error::$1</span>'
            }
        ];

        patterns.forEach(function(p) {
            html = html.replace(p.pattern, p.replacement);
        });
        
        block.innerHTML = html;
    }

    function addCodeBlockEnhancements() {
        document.querySelectorAll('pre code').forEach(function(block) {
            const pre = block.parentElement;
            
            // Add copy button
            const copyButton = document.createElement('button');
            copyButton.className = 'copy-button';
            copyButton.textContent = 'Copy';
            copyButton.onclick = function() {
                navigator.clipboard.writeText(block.textContent).then(function() {
                    copyButton.textContent = 'Copied!';
                    setTimeout(function() {
                        copyButton.textContent = 'Copy';
                    }, 2000);
                });
            };
            pre.style.position = 'relative';
            pre.appendChild(copyButton);

            // Add expand/collapse for long code blocks
            if (block.textContent.split('\n').length > 20) {
                pre.classList.add('code-expandable');
                
                const expandButton = document.createElement('div');
                expandButton.className = 'code-expand-button';
                expandButton.textContent = 'Show more...';
                expandButton.onclick = function() {
                    if (pre.classList.contains('expanded')) {
                        pre.classList.remove('expanded');
                        expandButton.textContent = 'Show more...';
                    } else {
                        pre.classList.add('expanded');
                        expandButton.textContent = 'Show less...';
                    }
                };
                pre.appendChild(expandButton);
            }

            // Add line numbers for Rust code
            if (block.classList.contains('language-rust')) {
                addLineNumbers(block);
            }

            // Add runnable playground button for complete examples
            if (isRunnableExample(block)) {
                const playButton = document.createElement('button');
                playButton.className = 'playground-button';
                playButton.textContent = '▶ Run';
                playButton.onclick = function() {
                    openInPlayground(block.textContent);
                };
                pre.appendChild(playButton);
            }
        });
    }

    function addLineNumbers(block) {
        const lines = block.textContent.split('\n');
        const lineNumbers = lines.map((_, i) => i + 1).join('\n');
        
        const lineNumbersEl = document.createElement('div');
        lineNumbersEl.className = 'line-numbers';
        lineNumbersEl.textContent = lineNumbers;
        
        block.parentElement.style.display = 'flex';
        block.parentElement.insertBefore(lineNumbersEl, block);
    }

    function isRunnableExample(block) {
        const code = block.textContent;
        return code.includes('fn main()') && 
               code.includes('pmcp::') &&
               !code.includes('// Not runnable') &&
               !code.includes('// Example only');
    }

    function openInPlayground(code) {
        // In a real implementation, this would integrate with Rust Playground
        // For now, we'll just log the code
        console.log('Would run in playground:', code);
        alert('Playground integration coming soon! Check the PMCP examples directory for runnable code.');
    }

    function setupPlaygroundIntegration() {
        // Add global keyboard shortcuts for code interaction
        document.addEventListener('keydown', function(e) {
            // Ctrl+Enter to run current code block
            if (e.ctrlKey && e.key === 'Enter') {
                const activeBlock = document.activeElement.closest('pre code');
                if (activeBlock && isRunnableExample(activeBlock)) {
                    openInPlayground(activeBlock.textContent);
                }
            }
        });

        // Add hover effects for interactive elements
        const style = document.createElement('style');
        style.textContent = `
            .pmcp-builder, .pmcp-method, .pmcp-transport, .pmcp-error {
                transition: background-color 0.2s;
                border-radius: 2px;
                padding: 1px 2px;
            }
            .pmcp-builder:hover { background-color: rgba(30, 136, 229, 0.2); }
            .pmcp-method:hover { background-color: rgba(76, 175, 80, 0.2); }
            .pmcp-transport:hover { background-color: rgba(255, 111, 0, 0.2); }
            .pmcp-error:hover { background-color: rgba(244, 67, 54, 0.2); }
        `;
        document.head.appendChild(style);
    }

    // Add protocol message formatting
    function formatProtocolMessages() {
        document.querySelectorAll('code.language-json').forEach(function(block) {
            const content = block.textContent;
            try {
                const obj = JSON.parse(content);
                if (obj.method || obj.result || obj.error) {
                    block.parentElement.classList.add('protocol-message');
                    
                    if (obj.method) {
                        block.parentElement.classList.add('protocol-request');
                    } else if (obj.result) {
                        block.parentElement.classList.add('protocol-response');
                    } else if (obj.error) {
                        block.parentElement.classList.add('protocol-error');
                    }
                }
            } catch (e) {
                // Not valid JSON, ignore
            }
        });
    }

    // Initialize protocol message formatting
    document.addEventListener('DOMContentLoaded', formatProtocolMessages);

    // Export for external use
    window.PMCPHighlight = {
        initializePMCPHighlighting,
        addCodeBlockEnhancements,
        formatProtocolMessages
    };
})();