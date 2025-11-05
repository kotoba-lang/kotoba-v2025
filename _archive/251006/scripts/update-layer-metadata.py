#!/usr/bin/env python3
"""
Update layer field in crate metadata.jsonld files based on new layer structure.
"""

import json
import os
from pathlib import Path

# Layer mapping: crate name -> new layer
LAYER_MAPPING = {
    # Foundation Layer (005-foundation)
    'kotoba-types': '005-foundation',
    'kotoba-cid': '005-foundation',
    'kotoba-schema': '005-foundation',
    'kotoba-auth': '005-foundation',
    'kotoba-graph-core': '005-foundation',
    'kotoba-logic': '005-foundation',
    
    # Logic Layer (010-logic)
    'kotoba-ir': '010-logic',
    'kotoba-rewrite-kernel': '010-logic',
    'kotoba-jsonld': '010-logic',
    'kotoba-codebase': '010-logic',
    'kotoba-txlog': '010-logic',
    'kotoba-api': '010-logic',
    'kotoba-phonosemantic': '010-logic',
    
    # VM Layer (012-vm)
    'kotoba-vm-core': '012-vm',
    'kotoba-vm-memory': '012-vm',
    'kotoba-vm-cpu': '012-vm',
    'kotoba-vm-scheduler': '012-vm',
    'kotoba-vm-gnn': '012-vm',
    'kotoba-vm-hardware': '012-vm',
    'kotoba-vm-types': '012-vm',
    
    # Reasoner Layer (014-reasoner)
    'kotoba-owl-reasoner': '014-reasoner',
    
    # OS Layer (015-os)
    'kotoba-os': '015-os',
    
    # Language Layer (020-language) - already correct
    'kotoba-syntax': '020-language',
    'kotoba-parser': '020-language',
    'kotoba-analyzer': '020-language',
    'kotoba-jsonnet': '020-language',
    'kotoba-kotobas': '020-language',
    'kotoba-formatter': '020-language',
    'kotoba-linter': '020-language',
    'kotoba-lsp': '020-language',
    'kotoba-repl': '020-language',
    'kotoba2tsx': '020-language',
    'kotobas-wasm': '020-language',
    'kotoba-language': '020-language',
    
    # Storage Layer (030-storage) - already correct
    'kotoba-storage': '030-storage',
    'kotoba-cache': '030-storage',
    'kotoba-db-cluster': '030-storage',
    'kotoba-distributed': '030-storage',
    'kotoba-graphdb': '030-storage',
    'kotoba-memory': '030-storage',
    'kotoba-storage-redis': '030-storage',
    'kotoba-storage-rocksdb': '030-storage',
    'kotoba-storage-fcdb': '030-storage',
    
    # Runtime Layer (040-runtime) - empty for now
    
    # Workflow Layer (050-workflow) - already correct
    'kotoba-workflow-core': '050-workflow',
    'kotoba-workflow': '050-workflow',
    'kotoba-workflow-activities': '050-workflow',
    'kotoba-workflow-operator': '050-workflow',
    
    # Application Layer (060-application)
    'kotoba-event-stream': '060-application',
    'kotoba-projection-engine': '060-application',
    'kotoba-rewrite': '060-application',
    'kotoba-query-engine': '060-application',
    'kotoba-execution': '060-application',
    'kotoba-handler': '060-application',
    'kotoba-routing': '060-application',
    'kotoba-state-graph': '060-application',
    
    # Services Layer (070-services) - already correct
    'kotoba-security': '070-services',
    'kotoba-network': '070-services',
    'kotoba-schema-registry': '070-services',
    'kotoba-server-core': '070-services',
    'kotoba-graph-api': '070-services',
    'kotoba-server-workflow': '070-services',
    'kotoba-server': '070-services',
    'kotoba-monitoring': '070-services',
    'kotoba-profiler': '070-services',
    'kotoba-cloud-integrations': '070-services',
    
    # Deployment Layer (080-deployment)
    'kotoba-deploy-core': '080-deployment',
    'kotoba-deploy': '080-deployment',
    'kotoba-deploy-scaling': '080-deployment',
    'kotoba-deploy-network': '080-deployment',
    'kotoba-deploy-git': '080-deployment',
    'kotoba-deploy-controller': '080-deployment',
    'kotoba-deploy-cli': '080-deployment',
    'kotoba-deploy-runtime': '080-deployment',
    'kotoba-deploy-hosting': '080-deployment',
    
    # Tools Layer (090-tools) - already correct
    'kotoba-config': '090-tools',
    'kotoba-build': '090-tools',
    'kotoba-package-manager': '090-tools',
    'kotoba-runtime': '090-tools',
    'kotoba-docs': '090-tools',
    'kotoba-ssg': '090-tools',
    'kotoba-tester': '090-tools',
    'kotoba-bench': '090-tools',
    'kotoba-backup': '090-tools',
    'kotoba-cli': '090-tools',
}

def update_metadata_file(metadata_path: Path):
    """Update layer field in a metadata.jsonld file."""
    try:
        with open(metadata_path, 'r', encoding='utf-8') as f:
            metadata = json.load(f)
        
        crate_name = metadata.get('name', '')
        if not crate_name:
            return False
        
        new_layer = LAYER_MAPPING.get(crate_name)
        if not new_layer:
            # Try to infer from path
            path_str = str(metadata_path)
            if '010-logic/020-kotoba-os' in path_str:
                new_layer = '015-os'
            elif '010-logic/022-kotoba-owl-reasoner' in path_str:
                new_layer = '014-reasoner'
            elif '005-vm' in path_str or 'kotoba-vm' in crate_name:
                new_layer = '012-vm'
            elif '010-logic' in path_str and new_layer is None:
                # Check if it's foundation or logic
                if crate_name in ['kotoba-types', 'kotoba-cid', 'kotoba-schema', 'kotoba-auth', 'kotoba-graph-core', 'kotoba-logic']:
                    new_layer = '005-foundation'
                else:
                    new_layer = '010-logic'
            elif '040-application' in path_str or '041-kotoba-event-stream' in crate_name or '042-kotoba-projection-engine' in crate_name or '043-kotoba-rewrite' in crate_name or '044-kotoba-query-engine' in crate_name or '045-kotoba-execution' in crate_name or '046-kotoba-handler' in crate_name or '047-kotoba-routing' in crate_name or '048-kotoba-state-graph' in crate_name:
                new_layer = '060-application'
            elif '030-storage' in path_str:
                new_layer = '030-storage'
            elif '020-language' in path_str:
                new_layer = '020-language'
            elif '050-workflow' in path_str:
                new_layer = '050-workflow'
            elif '060-services' in path_str:
                new_layer = '070-services'
            elif '070-deployment' in path_str:
                new_layer = '080-deployment'
            elif '090-tools' in path_str:
                new_layer = '090-tools'
            else:
                return False
        
        old_layer = metadata.get('layer', '').replace('kotoba:layer/', '')
        if old_layer != new_layer:
            metadata['layer'] = f'kotoba:layer/{new_layer}'
            
            with open(metadata_path, 'w', encoding='utf-8') as f:
                json.dump(metadata, f, indent=2, ensure_ascii=False)
            
            print(f"Updated {crate_name}: {old_layer} → {new_layer}")
            return True
        
        return False
    except Exception as e:
        print(f"Error updating {metadata_path}: {e}")
        return False

def main():
    """Main function."""
    workspace_root = Path(__file__).parent.parent
    crates_dir = workspace_root / "crates"
    
    if not crates_dir.exists():
        print(f"Error: {crates_dir} does not exist")
        return
    
    updated_count = 0
    for metadata_file in crates_dir.rglob("metadata.jsonld"):
        if update_metadata_file(metadata_file):
            updated_count += 1
    
    print(f"\nUpdated {updated_count} metadata files")

if __name__ == "__main__":
    main()

