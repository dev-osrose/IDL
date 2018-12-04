#pragma once


#include "packetfactory.h"

namespace RoseCommon {
namespace Packet {

class CliTest : public CRosePacket {
    public:
        CliTest();
        CliTest(CRoseReader reader);
        CliTest(CliTest&&) = default;
        CliTest& operator=(CliTest&&) = default;
        ~CliTest() = default;
        
        static constexpr size_t size();
        
        
        struct Test : public ISerialize {
            virtual bool read(CRoseReader&) override;
            virtual bool write(CRoseBasePolicy&) const override;
            
            static constexpr size_t size();
            
            void set_a(const int);
            int get_a() const;
            void set_b(const int);
            int get_b() const;
            
            private:
                union {
                    PACK(struct {
                        int a;
                    });
                    int b;
                } data;
        };
        
        
        
        
        static CliTest create();
        static CliTest create(const uint8_t*);
    
    protected:
        virtual void pack(CRoseBasePolicy&) const override;
    
    private:
};

}
}
