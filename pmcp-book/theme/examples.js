// PMCP Examples Interactive Features

(function() {
    'use strict';

    document.addEventListener('DOMContentLoaded', function() {
        initializeExampleFeatures();
        setupInteractiveElements();
        addQualityIndicators();
    });

    function initializeExampleFeatures() {
        // Add example metadata and interactivity
        document.querySelectorAll('pre code.language-rust').forEach(function(block) {
            const pre = block.parentElement;
            const metadata = extractExampleMetadata(block.textContent);
            
            if (metadata) {
                addExampleHeader(pre, metadata);
                addQualityBadges(pre, metadata);
                
                if (metadata.runnable) {
                    addRunButton(pre, block.textContent);
                }
                
                if (metadata.testable) {
                    addTestButton(pre, block.textContent);
                }
            }
        });
    }

    function extractExampleMetadata(code) {
        const metadata = {
            name: 'Example',
            runnable: false,
            testable: false,
            quality: 'good',
            features: [],
            complexity: 'basic'
        };

        // Check if it's a complete example
        if (code.includes('fn main()') && code.includes('#[tokio::main]')) {
            metadata.runnable = true;
            metadata.name = 'Complete Example';
        }

        // Check for test functions
        if (code.includes('#[test]') || code.includes('#[tokio::test]')) {
            metadata.testable = true;
            metadata.name = 'Test Example';
        }

        // Determine complexity
        const lines = code.split('\n').length;
        if (lines < 20) {
            metadata.complexity = 'basic';
        } else if (lines < 50) {
            metadata.complexity = 'intermediate';
        } else {
            metadata.complexity = 'advanced';
        }

        // Extract PMCP features used
        const featurePatterns = [
            { pattern: /Server::builder/, name: 'Server' },
            { pattern: /Client::builder/, name: 'Client' },
            { pattern: /ToolHandler/, name: 'Tools' },
            { pattern: /ResourceHandler/, name: 'Resources' },
            { pattern: /WebSocketTransport/, name: 'WebSocket' },
            { pattern: /HttpTransport/, name: 'HTTP' },
            { pattern: /StreamableHttp/, name: 'Streaming' },
            { pattern: /Authentication/, name: 'Auth' },
            { pattern: /Middleware/, name: 'Middleware' }
        ];

        featurePatterns.forEach(function(fp) {
            if (fp.pattern.test(code)) {
                metadata.features.push(fp.name);
            }
        });

        // Assess quality based on patterns
        const qualityChecks = [
            { pattern: /Error::(validation|internal|protocol)/, points: 2 },
            { pattern: /#\[test\]/, points: 2 },
            { pattern: /assert!/, points: 1 },
            { pattern: /tracing::(info|warn|error)/, points: 1 },
            { pattern: /\.unwrap\(\)/, points: -1 }, // Deduct for unwrap
            { pattern: /panic!/, points: -2 } // Deduct for panic
        ];

        let qualityScore = 5; // Start with neutral score
        qualityChecks.forEach(function(check) {
            if (check.pattern.test(code)) {
                qualityScore += check.points;
            }
        });

        if (qualityScore >= 8) {
            metadata.quality = 'excellent';
        } else if (qualityScore >= 6) {
            metadata.quality = 'good';
        } else if (qualityScore >= 4) {
            metadata.quality = 'fair';
        } else {
            metadata.quality = 'poor';
        }

        return metadata;
    }

    function addExampleHeader(pre, metadata) {
        const header = document.createElement('div');
        header.className = 'example-header';
        header.innerHTML = `
            <div class="example-title">${metadata.name}</div>
            <div class="example-meta">
                <span class="complexity-${metadata.complexity}">${metadata.complexity}</span>
                ${metadata.features.map(f => `<span class="feature-tag">${f}</span>`).join('')}
            </div>
        `;
        
        pre.parentElement.insertBefore(header, pre);
        
        // Add CSS for header styling
        if (!document.getElementById('example-header-styles')) {
            const style = document.createElement('style');
            style.id = 'example-header-styles';
            style.textContent = `
                .example-header {
                    background: linear-gradient(135deg, var(--pmcp-primary), var(--pmcp-primary-dark));
                    color: white;
                    padding: 12px 16px;
                    border-radius: 4px 4px 0 0;
                    margin-bottom: 0;
                    display: flex;
                    justify-content: space-between;
                    align-items: center;
                }
                .example-title {
                    font-weight: bold;
                    font-size: 1.1em;
                }
                .example-meta {
                    display: flex;
                    gap: 8px;
                    align-items: center;
                }
                .complexity-basic { background: #4CAF50; color: white; padding: 2px 8px; border-radius: 12px; font-size: 0.8em; }
                .complexity-intermediate { background: #FF9800; color: white; padding: 2px 8px; border-radius: 12px; font-size: 0.8em; }
                .complexity-advanced { background: #F44336; color: white; padding: 2px 8px; border-radius: 12px; font-size: 0.8em; }
                .feature-tag { 
                    background: rgba(255,255,255,0.2); 
                    color: white; 
                    padding: 2px 6px; 
                    border-radius: 8px; 
                    font-size: 0.7em; 
                }
            `;
            document.head.appendChild(style);
        }
    }

    function addQualityBadges(pre, metadata) {
        const qualityBadge = document.createElement('div');
        qualityBadge.className = `quality-badge quality-${metadata.quality}`;
        qualityBadge.title = `Code quality: ${metadata.quality}`;
        
        const icons = {
            excellent: '🏆',
            good: '✅', 
            fair: '⚠️',
            poor: '❌'
        };
        
        qualityBadge.innerHTML = `${icons[metadata.quality]} ${metadata.quality}`;
        qualityBadge.style.cssText = `
            position: absolute;
            top: 8px;
            right: 8px;
            z-index: 10;
            font-size: 0.8em;
            padding: 4px 8px;
            border-radius: 4px;
            background: rgba(0,0,0,0.8);
            color: white;
        `;
        
        pre.style.position = 'relative';
        pre.appendChild(qualityBadge);
    }

    function addRunButton(pre, code) {
        const runButton = document.createElement('button');
        runButton.className = 'example-run-button';
        runButton.innerHTML = '▶️ Run Example';
        runButton.style.cssText = `
            position: absolute;
            bottom: 8px;
            right: 8px;
            background: var(--pmcp-success);
            color: white;
            border: none;
            padding: 6px 12px;
            border-radius: 4px;
            cursor: pointer;
            font-size: 0.9em;
            opacity: 0;
            transition: opacity 0.2s;
        `;
        
        runButton.onclick = function() {
            runExample(code);
        };
        
        pre.addEventListener('mouseenter', function() {
            runButton.style.opacity = '1';
        });
        
        pre.addEventListener('mouseleave', function() {
            runButton.style.opacity = '0';
        });
        
        pre.appendChild(runButton);
    }

    function addTestButton(pre, code) {
        const testButton = document.createElement('button');
        testButton.className = 'example-test-button';
        testButton.innerHTML = '🧪 Run Tests';
        testButton.style.cssText = `
            position: absolute;
            bottom: 8px;
            right: 120px;
            background: var(--pmcp-warning);
            color: white;
            border: none;
            padding: 6px 12px;
            border-radius: 4px;
            cursor: pointer;
            font-size: 0.9em;
            opacity: 0;
            transition: opacity 0.2s;
        `;
        
        testButton.onclick = function() {
            runTests(code);
        };
        
        pre.addEventListener('mouseenter', function() {
            testButton.style.opacity = '1';
        });
        
        pre.addEventListener('mouseleave', function() {
            testButton.style.opacity = '0';
        });
        
        pre.appendChild(testButton);
    }

    function setupInteractiveElements() {
        // Add expandable sections for long examples
        document.querySelectorAll('.example-full').forEach(function(section) {
            const toggle = document.createElement('button');
            toggle.textContent = 'Show Full Example';
            toggle.className = 'example-toggle';
            toggle.onclick = function() {
                section.classList.toggle('expanded');
                toggle.textContent = section.classList.contains('expanded') 
                    ? 'Hide Full Example' 
                    : 'Show Full Example';
            };
            
            section.parentElement.insertBefore(toggle, section);
        });

        // Add tooltips for PMCP-specific concepts
        addConceptTooltips();
    }

    function addConceptTooltips() {
        const concepts = {
            'ToolHandler': 'A trait that defines how to handle tool execution requests',
            'RequestHandlerExtra': 'Extra context provided to request handlers',
            'ServerBuilder': 'Builder pattern for configuring MCP servers',
            'ClientBuilder': 'Builder pattern for configuring MCP clients',
            'ToolResult': 'Type alias for CallToolResult - the return type of tool handlers',
            'WebSocketTransport': 'Transport layer using WebSocket protocol',
            'StreamableHttp': 'HTTP transport with streaming capabilities'
        };

        Object.keys(concepts).forEach(function(concept) {
            const regex = new RegExp(`\\b${concept}\\b`, 'g');
            document.body.innerHTML = document.body.innerHTML.replace(
                regex,
                `<span class="pmcp-concept" title="${concepts[concept]}">${concept}</span>`
            );
        });

        // Style concept tooltips
        const style = document.createElement('style');
        style.textContent = `
            .pmcp-concept {
                border-bottom: 1px dotted var(--pmcp-primary);
                cursor: help;
                position: relative;
            }
            .pmcp-concept:hover::after {
                content: attr(title);
                position: absolute;
                bottom: 100%;
                left: 50%;
                transform: translateX(-50%);
                background: #333;
                color: white;
                padding: 8px 12px;
                border-radius: 4px;
                white-space: nowrap;
                z-index: 1000;
                font-size: 0.8em;
                box-shadow: 0 2px 8px rgba(0,0,0,0.3);
            }
        `;
        document.head.appendChild(style);
    }

    function addQualityIndicators() {
        // Add Toyota Way quality indicators
        document.querySelectorAll('.quality-gate').forEach(function(gate) {
            const indicator = document.createElement('span');
            indicator.className = 'quality-indicator';
            indicator.innerHTML = gate.classList.contains('passing') ? '✅' : '❌';
            gate.appendChild(indicator);
        });

        // Add test coverage indicators
        document.querySelectorAll('.coverage-info').forEach(function(info) {
            const percentage = parseFloat(info.textContent.match(/(\d+(?:\.\d+)?)%/)?.[1] || '0');
            const indicator = document.createElement('div');
            indicator.className = 'coverage-bar';
            indicator.innerHTML = `
                <div class="coverage-fill" style="width: ${percentage}%"></div>
                <span class="coverage-text">${percentage}%</span>
            `;
            info.appendChild(indicator);
        });

        // Style quality indicators
        const style = document.createElement('style');
        style.textContent = `
            .coverage-bar {
                width: 200px;
                height: 20px;
                background: #eee;
                border-radius: 10px;
                position: relative;
                margin-top: 4px;
                overflow: hidden;
            }
            .coverage-fill {
                height: 100%;
                background: linear-gradient(90deg, #f44336 0%, #ff9800 50%, #4caf50 100%);
                transition: width 0.5s ease;
            }
            .coverage-text {
                position: absolute;
                top: 50%;
                left: 50%;
                transform: translate(-50%, -50%);
                font-size: 0.8em;
                font-weight: bold;
                color: #333;
            }
        `;
        document.head.appendChild(style);
    }

    function runExample(code) {
        // In a real implementation, this would compile and run the code
        console.log('Running example:', code);
        
        // Show a modal with instructions for now
        showModal('Run Example', `
            <p>To run this example:</p>
            <ol>
                <li>Copy the code to a new Rust project</li>
                <li>Add PMCP dependencies to Cargo.toml</li>
                <li>Run with: <code>cargo run --features full</code></li>
            </ol>
            <p>Or check the examples/ directory in the PMCP repository for ready-to-run versions.</p>
        `);
    }

    function runTests(code) {
        console.log('Running tests:', code);
        showModal('Run Tests', `
            <p>To run these tests:</p>
            <ol>
                <li>Copy the code to your project's tests/ directory</li>
                <li>Run with: <code>cargo test</code></li>
            </ol>
        `);
    }

    function showModal(title, content) {
        const modal = document.createElement('div');
        modal.className = 'pmcp-modal';
        modal.innerHTML = `
            <div class="pmcp-modal-content">
                <div class="pmcp-modal-header">
                    <h3>${title}</h3>
                    <button class="pmcp-modal-close">&times;</button>
                </div>
                <div class="pmcp-modal-body">${content}</div>
            </div>
        `;

        modal.onclick = function(e) {
            if (e.target === modal) {
                document.body.removeChild(modal);
            }
        };

        modal.querySelector('.pmcp-modal-close').onclick = function() {
            document.body.removeChild(modal);
        };

        // Style modal
        modal.style.cssText = `
            position: fixed;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            background: rgba(0,0,0,0.5);
            display: flex;
            align-items: center;
            justify-content: center;
            z-index: 10000;
        `;

        const modalContent = modal.querySelector('.pmcp-modal-content');
        modalContent.style.cssText = `
            background: white;
            border-radius: 8px;
            max-width: 500px;
            width: 90%;
            max-height: 80vh;
            overflow-y: auto;
        `;

        const modalHeader = modal.querySelector('.pmcp-modal-header');
        modalHeader.style.cssText = `
            padding: 16px;
            border-bottom: 1px solid #eee;
            display: flex;
            justify-content: space-between;
            align-items: center;
        `;

        const modalBody = modal.querySelector('.pmcp-modal-body');
        modalBody.style.cssText = `
            padding: 16px;
        `;

        document.body.appendChild(modal);
    }

    // Export for external use
    window.PMCPExamples = {
        runExample,
        runTests,
        extractExampleMetadata,
        addQualityIndicators
    };
})();