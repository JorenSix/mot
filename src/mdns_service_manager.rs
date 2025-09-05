use mdns_sd::{ServiceDaemon, ServiceInfo, DaemonEvent};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

#[cfg(test)]
use std::thread;

/// A simple wrapper for mDNS service registration
pub struct MdnsService {
    daemon: Arc<ServiceDaemon>,
    service_fullname: Option<String>,
}

impl MdnsService {
    /// Create a new mDNS service instance
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let daemon = ServiceDaemon::new()?;
        Ok(Self {
            daemon: Arc::new(daemon),
            service_fullname: None,
        })
    }

    /// Register an mDNS service
    /// 
    /// # Arguments
    /// * `name` - The instance name of the service (e.g., "my-device")
    /// * `protocol` - The service protocol (e.g., "_http._tcp" or "_ssh._tcp")
    /// * `port` - The port number the service is running on
    /// 
    /// # Example
    /// ```
    /// let mut mdns = MdnsService::new()?;
    /// mdns.register("my-device", "_http._tcp", 8080)?;
    /// ```
    pub fn register(
        &mut self,
        name: &str,
        protocol: &str,
        port: u16,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Format the service type with .local. suffix
        let service_type = if protocol.ends_with(".local.") {
            protocol.to_string()
        } else {
            format!("{}.local.", protocol)
        };

        // Use the machine's hostname or a default
        let hostname = hostname::get()
            .unwrap_or_else(|_| "localhost".into())
            .to_string_lossy()
            .to_string();
        let service_hostname = format!("{}.local.", hostname);

        // Create service info with auto-discovered addresses
        let properties: [(&str, &str); 0] = [];
        let service_info = ServiceInfo::new(
            &service_type,
            name,
            &service_hostname,
            "", // Empty string for auto-discovery
            port,
            &properties[..],
        )?
        .enable_addr_auto();

        // Store the fullname for potential unregistration
        self.service_fullname = Some(service_info.get_fullname().to_string());

        // Register the service
        self.daemon.register(service_info)?;

        Ok(())
    }

    /// Unregister the currently registered service
    pub fn unregister(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(fullname) = &self.service_fullname {
            let receiver = self.daemon.unregister(fullname)?;
            
            // Wait for unregistration to complete
            while let Ok(_event) = receiver.recv() {
                // Process events until channel closes
            }
            
            self.service_fullname = None;
        }
        Ok(())
    }

    /// Keep the service running while monitoring the running flag
    /// Will automatically unregister when running becomes false
    pub fn run_with_interrupt(
        &mut self, 
        running: Arc<AtomicBool>
    ) -> Result<(), Box<dyn std::error::Error>> {
        let monitor = self.daemon.monitor()?;
        
        // Set a receive timeout so we can periodically check the running flag
        let timeout = Duration::from_millis(100);
        
        while running.load(Ordering::SeqCst) {
            match monitor.recv_timeout(timeout) {
                Ok(event) => {
                    if let DaemonEvent::Error(e) = event {
                        // Unregister before returning error
                        let _ = self.unregister();
                        return Err(Box::new(e));
                    }
                }
                Err(flume::RecvTimeoutError::Timeout) => {
                    // Timeout occurred, check running flag and continue
                    continue;
                }
                Err(flume::RecvTimeoutError::Disconnected) => {
                    // Channel disconnected, exit gracefully
                    break;
                }
            }
        }
        
        // Unregister the service when stopping
        self.unregister()?;

        #[cfg(debug_assertions)]
        println!("mDNS service unregistered gracefully");
        
        Ok(())
    }

}

impl Drop for MdnsService {
    fn drop(&mut self) {
        // Attempt to unregister on drop
        let _ = self.unregister();
    }
}

// Example usage
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::thread;

    #[test]
    fn test_interrupt_registration() {
        let mut mdns = MdnsService::new().unwrap();
        let running = Arc::new(AtomicBool::new(true));
        
        // Register a simple HTTP service
        mdns.register("my-web-server", "_http._tcp", 8080).unwrap();
        
        // Simulate interrupt after 2 seconds
        let running_clone = running.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_secs(2));
            running_clone.store(false, Ordering::SeqCst);
        });
        
        // This should stop when running becomes false
        mdns.run_with_interrupt(running).unwrap();
    }
}
